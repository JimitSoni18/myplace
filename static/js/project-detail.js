/**
 * MyPlace - Project Detail Showcase Controller
 * Loaded ONLY on /projects/{slug}
 */

// Fullscreen Media Gallery & Lightbox
const mediaGallery = {
	currentIndex: 0,
	items: [],
	isOpen: false,
	init(mediaItems) {
		this.items = mediaItems || [];
	},
	open(index = 0) {
		this.currentIndex = index;
		this.isOpen = true;
		const modal = document.getElementById('gallery-lightbox-modal');
		if (modal) {
			modal.style.display = 'flex';
			document.body.style.overflow = 'hidden';
			this.render();
		}
	},
	close() {
		this.isOpen = false;
		const modal = document.getElementById('gallery-lightbox-modal');
		if (modal) {
			modal.style.display = 'none';
			document.body.style.overflow = '';
		}
	},
	next() {
		if (this.items.length > 0) {
			this.currentIndex = (this.currentIndex + 1) % this.items.length;
			this.render();
		}
	},
	prev() {
		if (this.items.length > 0) {
			this.currentIndex = (this.currentIndex - 1 + this.items.length) % this.items.length;
			this.render();
		}
	},
	render() {
		const stage = document.getElementById('lightbox-active-stage');
		const counter = document.getElementById('lightbox-counter-display');
		if (!stage || !this.items[this.currentIndex]) return;
		const item = this.items[this.currentIndex];
		if (item.type === 'video') {
			stage.innerHTML = `<video controls autoplay style="max-height:85vh; max-width:90vw; border-radius:12px;"><source src="${item.url}" type="video/mp4"></video>`;
		} else {
			stage.innerHTML = `<img src="${item.url}" alt="Gallery photo ${this.currentIndex + 1}" style="max-height:85vh; max-width:90vw; object-fit:contain; border-radius:12px;">`;
		}
		if (counter) {
			counter.textContent = `${this.currentIndex + 1} of ${this.items.length}`;
		}
	}
};

// Floor Plan Modal
function openFloorPlanModal(imageUrl, title) {
	let modal = document.getElementById('floor-plan-lightbox-modal');
	if (!modal) {
		modal = document.createElement('div');
		modal.id = 'floor-plan-lightbox-modal';
		modal.className = 'lightbox-modal';
		modal.style.display = 'none';
		modal.innerHTML = `
			<button class="lightbox-close-btn" onclick="closeFloorPlanModal()" aria-label="Close Floor Plan">
				<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
					<line x1="18" y1="6" x2="6" y2="18"></line>
					<line x1="6" y1="6" x2="18" y2="18"></line>
				</svg>
			</button>
			<div class="lightbox-content-box" style="flex-direction:column; gap:1rem;">
				<div id="fp-modal-title" style="color:white; font-size:1.2rem; font-weight:700; text-align:center;"></div>
				<img id="fp-modal-img" src="" alt="Floor Plan" style="background:white; padding:1.5rem; border-radius:12px; max-height:80vh; max-width:90vw; object-fit:contain;">
			</div>
		`;
		modal.addEventListener('click', (e) => {
			if (e.target === modal) closeFloorPlanModal();
		});
		document.body.appendChild(modal);
	}
	document.getElementById('fp-modal-title').textContent = title || 'Floor Plan Layout';
	document.getElementById('fp-modal-img').src = imageUrl;
	modal.style.display = 'flex';
	document.body.style.overflow = 'hidden';
}

function closeFloorPlanModal() {
	const modal = document.getElementById('floor-plan-lightbox-modal');
	if (modal) {
		modal.style.display = 'none';
		document.body.style.overflow = '';
	}
}

// Typology BHK Tab Switcher
function filterTypologies(bhkFilter) {
	const buttons = document.querySelectorAll('.typology-tab-btn');
	buttons.forEach(btn => {
		if (btn.getAttribute('data-filter') === bhkFilter) {
			btn.classList.add('active');
		} else {
			btn.classList.remove('active');
		}
	});

	const cards = document.querySelectorAll('.typology-card');
	cards.forEach(card => {
		const cardBhk = card.getAttribute('data-bhk');
		if (bhkFilter === 'all' || cardBhk === bhkFilter) {
			card.style.display = 'flex';
		} else {
			card.style.display = 'none';
		}
	});
}

// Connectivity Hub Category Switcher
function filterConnectivity(catFilter) {
	const buttons = document.querySelectorAll('.connectivity-tab-btn');
	buttons.forEach(btn => {
		if (btn.getAttribute('data-filter') === catFilter) {
			btn.classList.add('active');
		} else {
			btn.classList.remove('active');
		}
	});

	const cards = document.querySelectorAll('.landmark-card');
	cards.forEach(card => {
		const cardCat = card.getAttribute('data-category');
		if (catFilter === 'all' || cardCat === catFilter) {
			card.style.display = 'flex';
		} else {
			card.style.display = 'none';
		}
	});
}

// Specifications Accordion Controller
function toggleSpecAccordion(headerEl) {
	const item = headerEl.closest('.spec-accordion-item');
	if (!item) return;
	const wasActive = item.classList.contains('active');
	item.classList.toggle('active', !wasActive);
}

// Lead form configuration selector
function selectConfigOption(configId) {
	const sel = document.getElementById('configuration_id');
	if (sel) {
		sel.value = configId;
	}
	const enquirySec = document.getElementById('enquiry');
	if (enquirySec) {
		enquirySec.scrollIntoView({ behavior: 'smooth', block: 'start' });
	}
}

// Keyboard shortcuts for Lightboxes
window.addEventListener('keydown', (e) => {
	if (mediaGallery.isOpen) {
		if (e.key === 'Escape') mediaGallery.close();
		if (e.key === 'ArrowRight') mediaGallery.next();
		if (e.key === 'ArrowLeft') mediaGallery.prev();
	}
	const fpModal = document.getElementById('floor-plan-lightbox-modal');
	if (fpModal && fpModal.style.display !== 'none') {
		if (e.key === 'Escape') closeFloorPlanModal();
	}
});

// Scroll-Spy for Sticky Anchor Sub-Nav
document.addEventListener('DOMContentLoaded', () => {
	const subNavLinks = document.querySelectorAll('.sub-nav-pill');
	if (subNavLinks.length > 0) {
		const sections = [];
		subNavLinks.forEach(link => {
			const targetId = link.getAttribute('href');
			if (targetId && targetId.startsWith('#')) {
				const sec = document.querySelector(targetId);
				if (sec) sections.push({ link, sec });
			}
		});

		window.addEventListener('scroll', () => {
			const scrollPos = window.scrollY + 140;
			let activeFound = false;
			for (let i = sections.length - 1; i >= 0; i--) {
				const { link, sec } = sections[i];
				if (sec.offsetTop <= scrollPos) {
					subNavLinks.forEach(l => l.classList.remove('active'));
					link.classList.add('active');
					activeFound = true;
					break;
				}
			}
			if (!activeFound && subNavLinks[0]) {
				subNavLinks.forEach(l => l.classList.remove('active'));
				subNavLinks[0].classList.add('active');
			}
		}, { passive: true });
	}
});
