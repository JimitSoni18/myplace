use std::{collections::BTreeMap, ops::Deref, sync::Mutex};

use tokio::sync::RwLock;
use uuid::{ClockSequence, ContextV7, Timestamp, Uuid};

use crate::constants::misc::SIX_HOURS_SECONDS;

// ---------------------------------------------------------------------------
// UUID v7 shared clock context
// ---------------------------------------------------------------------------

pub struct SharedContextV7(Mutex<ContextV7>);

pub fn get_shared_context() -> &'static SharedContextV7 {
	static CONTEXT_V7: SharedContextV7 = SharedContextV7(Mutex::new(ContextV7::new()));
	&CONTEXT_V7
}

impl ClockSequence for SharedContextV7 {
	type Output = u64;

	fn generate_sequence(&self, seconds: u64, subsec_nanos: u32) -> Self::Output {
		self.0.generate_sequence(seconds, subsec_nanos)
	}

	fn generate_timestamp_sequence(
		&self,
		seconds: u64,
		subsec_nanos: u32,
	) -> (Self::Output, u64, u32) {
		self.0.generate_timestamp_sequence(seconds, subsec_nanos)
	}

	fn usable_bits(&self) -> usize
	where
		Self::Output: Sized,
	{
		42
	}
}

// ---------------------------------------------------------------------------
// User role
// ---------------------------------------------------------------------------

/// Distinguishes authenticated principals so that a single session store can
/// serve both admin users and project owners without mixing up authorization.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UserRole {
	Admin,
	ProjectOwner,
}

// ---------------------------------------------------------------------------
// Session data
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct AuthUser {
	pub user_id: i32,
	pub username: String,
	pub role: UserRole,
	pub started_at: Timestamp,
	pub expires_at: Timestamp,
	// TODO: add client platform: ip, browser, location, etc.
}

impl AuthUser {
	pub fn new(user_id: i32, username: String, role: UserRole) -> Self {
		let shared_context = get_shared_context();
		let started_at = Timestamp::now(shared_context);
		let (seconds, subsec_nanos) = started_at.to_unix();
		let expires_at =
			Timestamp::from_unix(shared_context, seconds + SIX_HOURS_SECONDS, subsec_nanos);

		Self {
			user_id,
			username,
			role,
			started_at,
			expires_at,
		}
	}

	pub fn with_creation_timestamp(self, started_at: Timestamp) -> Self {
		Self { started_at, ..self }
	}

	/// Returns `true` if this session has passed its `expires_at` time.
	pub fn is_expired(&self) -> bool {
		let (expires_at_secs, _) = self.expires_at.to_unix();
		let (now_secs, _) = Timestamp::now(get_shared_context()).to_unix();
		// Expired when the deadline is in the past.
		expires_at_secs < now_secs
	}
}

// ---------------------------------------------------------------------------
// Session ID
// ---------------------------------------------------------------------------

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct SessionId(pub Uuid);

impl Deref for SessionId {
	type Target = Uuid;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

// ---------------------------------------------------------------------------
// SessionStore trait
// ---------------------------------------------------------------------------

/// An abstraction over session storage. The in-memory implementation is used
/// today; this trait makes it easy to swap in a Redis-backed implementation
/// later without touching any handlers.
pub trait SessionStoreTrait: Send + Sync {
	async fn create_session(&self, auth_user: AuthUser) -> SessionId;
	/// Returns the session if it exists **and has not expired**.
	async fn get_session(&self, session_id: &SessionId) -> Option<AuthUser>;
	async fn delete_session(&self, session_id: &SessionId) -> bool;
	async fn delete_all_for_user(&self, user_id: i32);
	/// Rotates the session ID; the new ID keeps the original `started_at`.
	async fn renew_session(
		&self,
		session_id: &SessionId,
	) -> Result<SessionId, SessionRenewError>;
	/// Removes all expired sessions. Returns the number removed.
	async fn clean_expired(&self) -> usize;
}

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

pub enum SessionRenewError {
	SessionNotFound,
}

// ---------------------------------------------------------------------------
// In-memory implementation
// ---------------------------------------------------------------------------

#[derive(Default)]
pub struct SessionStore {
	sessions: RwLock<BTreeMap<SessionId, AuthUser>>,
}

impl SessionStore {
	fn new_session_id() -> SessionId {
		SessionId(Uuid::new_v7(Timestamp::now(get_shared_context())))
	}
}

impl SessionStoreTrait for SessionStore {
	async fn create_session(&self, auth_user: AuthUser) -> SessionId {
		let session_id = Self::new_session_id();
		self.sessions.write().await.insert(session_id, auth_user);
		session_id
	}

	async fn get_session(&self, session_id: &SessionId) -> Option<AuthUser> {
		let guard = self.sessions.read().await;
		let user = guard.get(session_id)?;
		if user.is_expired() {
			return None;
		}
		Some(user.clone())
	}

	async fn delete_session(&self, session_id: &SessionId) -> bool {
		self.sessions.write().await.remove(session_id).is_some()
	}

	async fn delete_all_for_user(&self, user_id: i32) {
		self.sessions
			.write()
			.await
			.retain(|_, v| v.user_id != user_id);
	}

	/// Rotates the session ID preserving the original `started_at` so session
	/// age limits can be enforced later if needed.
	async fn renew_session(
		&self,
		session_id: &SessionId,
	) -> Result<SessionId, SessionRenewError> {
		let mut guard = self.sessions.write().await;
		let AuthUser {
			started_at,
			user_id,
			username,
			role,
			..
		} = guard
			.remove(session_id)
			.ok_or(SessionRenewError::SessionNotFound)?;

		let new_auth_user =
			AuthUser::new(user_id, username, role).with_creation_timestamp(started_at);
		let new_session_id = Self::new_session_id();
		// Discard previous value — a collision here is practically impossible with UUIDv7.
		guard.insert(new_session_id, new_auth_user);
		Ok(new_session_id)
	}

	async fn clean_expired(&self) -> usize {
		let mut guard = self.sessions.write().await;
		let before = guard.len();
		guard.retain(|_, user| !user.is_expired());
		before - guard.len()
	}
}
