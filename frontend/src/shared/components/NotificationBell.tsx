import { createSignal, onMount, onCleanup, For, Show, createMemo } from "solid-js";
import {
  Bell,
  CheckCheck,
  Clock,
  Mail,
  Phone,
  X,
  Building2,
  Home,
  Check,
  Search,
  ChevronDown,
  ChevronUp,
  Inbox,
  ExternalLink,
} from "lucide-solid";
import { api, getToken, BASE_URL } from "@/shared/api/client";
import { Modal } from "@/shared/components/Modal";
import { Button } from "@/shared/components/Button";
import type { EnquiryNotification, NotificationsListResponse } from "@/shared/api/types";
import { debounce } from "@/shared/utils/debounce";

interface NotificationBellProps {
  endpointPrefix: string;
}

export function NotificationBell(props: NotificationBellProps) {
  const [notifications, setNotifications] = createSignal<EnquiryNotification[]>([]);
  const [unreadCount, setUnreadCount] = createSignal(0);
  const [isOpen, setIsOpen] = createSignal(false);
  const [activeTab, setActiveTab] = createSignal<"unread" | "all">("unread");
  const [toast, setToast] = createSignal<{ title: string; message: string } | null>(null);

  // Full inbox modal state
  const [inboxOpen, setInboxOpen] = createSignal(false);
  const [inboxSearch, setInboxSearch] = createSignal("");
  const [debouncedInboxSearch, setDebouncedInboxSearch] = createSignal("");
  const updateDebouncedInboxSearch = debounce((val: string) => {
    setDebouncedInboxSearch(val);
    setInboxPage(1);
  }, 250);
  const [inboxTypeFilter, setInboxTypeFilter] = createSignal("all");
  const [inboxStatusFilter, setInboxStatusFilter] = createSignal("all");
  const [inboxPage, setInboxPage] = createSignal(1);
  const PAGE_SIZE = 8;

  // Expanded message IDs
  const [expandedIds, setExpandedIds] = createSignal<Set<string>>(new Set());

  const toggleExpand = (key: string) => {
    setExpandedIds((prev) => {
      const next = new Set(prev);
      if (next.has(key)) next.delete(key);
      else next.add(key);
      return next;
    });
  };

  let dropdownRef: HTMLDivElement | undefined;
  let toastTimer: ReturnType<typeof setTimeout> | undefined;

  const showToast = (title: string, message: string) => {
    if (toastTimer) clearTimeout(toastTimer);
    setToast({ title, message });
    toastTimer = setTimeout(() => {
      setToast(null);
    }, 5000);
  };

  const loadNotifications = async () => {
    try {
      const data = await api.get<NotificationsListResponse>(props.endpointPrefix);
      setNotifications(data.items || []);
      setUnreadCount(data.unread_count || 0);
    } catch (err) {
      console.error("Failed to load notifications:", err);
    }
  };

  const markAsRead = async (item: EnquiryNotification) => {
    if (item.is_read) return;
    setNotifications((prev) =>
      prev.map((n) =>
        n.id === item.id && n.enquiry_type === item.enquiry_type ? { ...n, is_read: true } : n
      )
    );
    setUnreadCount((c) => Math.max(0, c - 1));

    try {
      await api.post(`${props.endpointPrefix}/${item.enquiry_type}/${item.id}/read`);
    } catch (err) {
      console.error("Failed to mark notification as read:", err);
    }
  };

  const markAllAsRead = async () => {
    if (unreadCount() === 0) return;
    setNotifications((prev) => prev.map((n) => ({ ...n, is_read: true })));
    setUnreadCount(0);

    try {
      await api.post(`${props.endpointPrefix}/read-all`);
    } catch (err) {
      console.error("Failed to mark all as read:", err);
    }
  };

  onMount(() => {
    loadNotifications();

    // Setup SSE real-time stream with authentication token
    const token = getToken();
    const streamEndpoint = `${BASE_URL}${props.endpointPrefix}/stream`;
    const sseUrl = token
      ? `${streamEndpoint}?token=${encodeURIComponent(token)}`
      : streamEndpoint;

    const eventSource = new EventSource(sseUrl, { withCredentials: true });

    const handleIncoming = (e: MessageEvent) => {
      try {
        const notif: EnquiryNotification = JSON.parse(e.data);
        setNotifications((prev) => [
          notif,
          ...prev.filter((n) => !(n.id === notif.id && n.enquiry_type === notif.enquiry_type)),
        ]);
        if (!notif.is_read) {
          setUnreadCount((c) => c + 1);
        }
        showToast(
          `New ${notif.enquiry_type === "property" ? "Unit" : "Project"} Enquiry`,
          `${notif.name} inquired for "${notif.entity_name}"`
        );
      } catch (err) {
        console.error("Error parsing incoming notification event:", err);
      }
    };

    eventSource.addEventListener("notification", handleIncoming);
    eventSource.addEventListener("enquiry", handleIncoming);
    eventSource.onmessage = handleIncoming;

    eventSource.onerror = () => {
      // EventSource automatically retries
    };

    const handleClickOutside = (e: MouseEvent) => {
      if (dropdownRef && !dropdownRef.contains(e.target as Node)) {
        setIsOpen(false);
      }
    };
    document.addEventListener("mousedown", handleClickOutside);

    onCleanup(() => {
      eventSource.close();
      document.removeEventListener("mousedown", handleClickOutside);
      if (toastTimer) clearTimeout(toastTimer);
    });
  });

  const parseDate = (dateStr: string): Date | null => {
    if (!dateStr) return null;
    let s = dateStr.trim();
    if (s.includes(" ") && !s.includes("T")) {
      s = s.replace(" ", "T");
    }
    const d = new Date(s);
    return isNaN(d.getTime()) ? null : d;
  };

  const formatTimestamp = (dateStr: string): string => {
    const d = parseDate(dateStr);
    if (!d) return "Recently";
    const now = new Date();
    const diffMs = now.getTime() - d.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    if (diffMins < 1) return "Just now";
    if (diffMins < 60) return `${diffMins}m ago`;
    const diffHours = Math.floor(diffMins / 60);
    if (diffHours < 24) return `${diffHours}h ago`;
    const diffDays = Math.floor(diffHours / 24);
    if (diffDays === 1) return "Yesterday";
    if (diffDays < 7) return `${diffDays}d ago`;
    return d.toLocaleDateString(undefined, { month: "short", day: "numeric" });
  };

  // Filtered list for the quick flyout
  const flyoutNotifications = createMemo(() => {
    const list = notifications();
    if (activeTab() === "unread") {
      return list.filter((n) => !n.is_read).slice(0, 8);
    }
    return list.slice(0, 8);
  });

  // Filtered & paginated list for the full inbox modal
  const filteredInboxItems = createMemo(() => {
    const search = debouncedInboxSearch().toLowerCase().trim();
    const typeFilter = inboxTypeFilter();
    const statusFilter = inboxStatusFilter();

    return notifications().filter((n) => {
      const matchSearch =
        !search ||
        n.name.toLowerCase().includes(search) ||
        n.phone.includes(search) ||
        (n.email && n.email.toLowerCase().includes(search)) ||
        n.entity_name.toLowerCase().includes(search) ||
        (n.message && n.message.toLowerCase().includes(search));

      const matchType = typeFilter === "all" || n.enquiry_type === typeFilter;
      const matchStatus =
        statusFilter === "all" ||
        (statusFilter === "unread" && !n.is_read) ||
        (statusFilter === "read" && n.is_read);

      return matchSearch && matchType && matchStatus;
    });
  });

  const paginatedInboxItems = createMemo(() => {
    const start = (inboxPage() - 1) * PAGE_SIZE;
    return filteredInboxItems().slice(start, start + PAGE_SIZE);
  });

  const totalInboxPages = createMemo(() =>
    Math.max(1, Math.ceil(filteredInboxItems().length / PAGE_SIZE))
  );

  return (
    <div class="relative" ref={dropdownRef}>
      {/* Bell Trigger Button */}
      <button
        onClick={() => setIsOpen(!isOpen())}
        title="Enquiry Notifications"
        class="relative p-2 text-slate-400 hover:text-slate-100 hover:bg-slate-800 rounded-lg transition-colors cursor-pointer focus:outline-none focus-visible:ring-2 focus-visible:ring-indigo-500"
      >
        <Bell class="w-5 h-5" />
        <Show when={unreadCount() > 0}>
          <span class="absolute top-1 right-1 flex items-center justify-center min-w-[18px] h-[18px] px-1 text-[10px] font-bold text-white bg-rose-600 rounded-full border-2 border-slate-900 shadow-sm animate-pulse">
            {unreadCount() > 99 ? "99+" : unreadCount()}
          </span>
        </Show>
      </button>

      {/* Quick Flyout Dropdown */}
      <Show when={isOpen()}>
        <div class="absolute right-0 mt-2 w-80 sm:w-96 max-h-[85vh] flex flex-col bg-slate-900 border border-slate-800 rounded-2xl shadow-2xl shadow-black/80 z-50 overflow-hidden animate-in fade-in slide-in-from-top-2 duration-150">
          {/* Header */}
          <div class="p-3.5 border-b border-slate-800 bg-slate-900/95 backdrop-blur-md">
            <div class="flex items-center justify-between mb-2">
              <div class="flex items-center gap-2">
                <span class="text-sm font-bold text-white">Buyer Inquiries</span>
                <Show when={unreadCount() > 0}>
                  <span class="px-2 py-0.5 text-[10px] font-bold rounded-full bg-rose-500/20 text-rose-400 border border-rose-500/30">
                    {unreadCount()} new
                  </span>
                </Show>
              </div>
              <Show when={unreadCount() > 0}>
                <button
                  onClick={markAllAsRead}
                  class="flex items-center gap-1 text-[11px] text-indigo-400 hover:text-indigo-300 font-medium transition-colors cursor-pointer"
                >
                  <CheckCheck class="w-3.5 h-3.5" />
                  Mark all read
                </button>
              </Show>
            </div>

            {/* Quick Filter Tabs */}
            <div class="flex items-center gap-1 p-0.5 bg-slate-950 rounded-lg border border-slate-800/80">
              <button
                onClick={() => setActiveTab("unread")}
                class={`flex-1 py-1 text-xs font-semibold rounded-md transition-all cursor-pointer ${
                  activeTab() === "unread"
                    ? "bg-slate-800 text-white shadow-sm"
                    : "text-slate-400 hover:text-slate-200"
                }`}
              >
                Unread ({unreadCount()})
              </button>
              <button
                onClick={() => setActiveTab("all")}
                class={`flex-1 py-1 text-xs font-semibold rounded-md transition-all cursor-pointer ${
                  activeTab() === "all"
                    ? "bg-slate-800 text-white shadow-sm"
                    : "text-slate-400 hover:text-slate-200"
                }`}
              >
                All Recent ({notifications().length})
              </button>
            </div>
          </div>

          {/* Notifications List */}
          <div class="flex-1 overflow-y-auto divide-y divide-slate-800/60 max-h-96">
            <Show
              when={flyoutNotifications().length > 0}
              fallback={
                <div class="py-12 px-4 text-center text-slate-500">
                  <Inbox class="w-8 h-8 mx-auto mb-2 opacity-30 text-slate-400" />
                  <p class="text-xs font-medium text-slate-300">
                    {activeTab() === "unread" ? "All caught up!" : "No inquiries yet"}
                  </p>
                  <p class="text-[11px] text-slate-500 mt-0.5">
                    {activeTab() === "unread"
                      ? "There are no unread inquiries waiting for your review."
                      : "Prospective buyer inquiries will appear here in real time."}
                  </p>
                </div>
              }
            >
              <For each={flyoutNotifications()}>
                {(item) => {
                  const itemKey = `${item.enquiry_type}-${item.id}`;
                  const isExpanded = () => expandedIds().has(itemKey);
                  const isProperty = item.enquiry_type === "property";

                  return (
                    <div
                      class={`p-3.5 transition-colors ${
                        !item.is_read ? "bg-slate-850/60 hover:bg-slate-850" : "hover:bg-slate-850/40 opacity-85"
                      }`}
                    >
                      <div class="flex items-start justify-between gap-2.5">
                        {/* Customer Avatar & Details */}
                        <div class="flex items-start gap-2.5 flex-1 min-w-0">
                          <div
                            class={`w-8 h-8 rounded-full flex items-center justify-center font-bold text-xs text-white shrink-0 shadow-sm ${
                              isProperty
                                ? "bg-gradient-to-tr from-emerald-600 to-teal-500"
                                : "bg-gradient-to-tr from-indigo-600 to-indigo-400"
                            }`}
                          >
                            {item.name.charAt(0).toUpperCase()}
                          </div>

                          <div class="flex-1 min-w-0">
                            <div class="flex items-center gap-1.5 flex-wrap">
                              <span class="text-xs font-semibold text-slate-100 truncate">
                                {item.name}
                              </span>
                              <span
                                class={`px-1.5 py-0.5 text-[9px] font-semibold uppercase tracking-wider rounded ${
                                  isProperty
                                    ? "bg-emerald-950/70 text-emerald-300 border border-emerald-800/60"
                                    : "bg-indigo-950/70 text-indigo-300 border border-indigo-800/60"
                                }`}
                              >
                                {isProperty ? "Unit" : "Project"}
                              </span>
                            </div>

                            <div class="text-[11px] text-slate-300 font-medium flex items-center gap-1 mt-0.5">
                              {isProperty ? (
                                <Home class="w-3 h-3 text-emerald-400 shrink-0" />
                              ) : (
                                <Building2 class="w-3 h-3 text-indigo-400 shrink-0" />
                              )}
                              <span class="truncate">{item.entity_name}</span>
                            </div>
                          </div>
                        </div>

                        {/* Relative Timestamp & Mark Read action */}
                        <div class="flex flex-col items-end gap-1 shrink-0">
                          <span class="text-[10px] text-slate-500 flex items-center gap-1 font-mono">
                            <Clock class="w-2.5 h-2.5" />
                            {formatTimestamp(item.created_at)}
                          </span>

                          <Show when={!item.is_read}>
                            <button
                              onClick={() => markAsRead(item)}
                              title="Mark as read"
                              class="text-[10px] text-indigo-400 hover:text-indigo-300 flex items-center gap-0.5 cursor-pointer"
                            >
                              <Check class="w-3 h-3" /> Mark read
                            </button>
                          </Show>
                        </div>
                      </div>

                      {/* Contact Channels */}
                      <div class="mt-2.5 flex items-center gap-2">
                        <a
                          href={`tel:${item.phone}`}
                          class="inline-flex items-center gap-1 px-2.5 py-1 rounded-md text-[11px] font-semibold bg-slate-800 hover:bg-slate-750 text-slate-200 hover:text-white border border-slate-700/80 transition-colors"
                        >
                          <Phone class="w-3 h-3 text-emerald-400" />
                          {item.phone}
                        </a>
                        <Show when={item.email}>
                          <a
                            href={`mailto:${item.email}`}
                            class="inline-flex items-center gap-1 px-2.5 py-1 rounded-md text-[11px] font-semibold bg-slate-800 hover:bg-slate-750 text-slate-200 hover:text-white border border-slate-700/80 transition-colors truncate max-w-[160px]"
                          >
                            <Mail class="w-3 h-3 text-indigo-400 shrink-0" />
                            <span class="truncate">{item.email}</span>
                          </a>
                        </Show>
                      </div>

                      {/* Message Preview Accordion */}
                      <Show when={item.message}>
                        <div class="mt-2 pt-2 border-t border-slate-800/50">
                          <p
                            onClick={() => toggleExpand(itemKey)}
                            class={`text-[11px] text-slate-300 cursor-pointer ${
                              isExpanded() ? "whitespace-pre-line" : "line-clamp-2"
                            }`}
                          >
                            "{item.message}"
                          </p>
                          <button
                            onClick={() => toggleExpand(itemKey)}
                            class="text-[10px] text-indigo-400 hover:text-indigo-300 font-medium mt-0.5 flex items-center gap-0.5 cursor-pointer"
                          >
                            {isExpanded() ? (
                              <>
                                Show less <ChevronUp class="w-2.5 h-2.5" />
                              </>
                            ) : (
                              <>
                                View full inquiry message <ChevronDown class="w-2.5 h-2.5" />
                              </>
                            )}
                          </button>
                        </div>
                      </Show>
                    </div>
                  );
                }}
              </For>
            </Show>
          </div>

          {/* Footer: Open Full Inbox */}
          <div class="p-3 border-t border-slate-800 bg-slate-900/90 flex items-center justify-between">
            <span class="text-[11px] text-slate-400">
              {notifications().length} total recorded
            </span>
            <button
              onClick={() => {
                setIsOpen(false);
                setInboxOpen(true);
              }}
              class="text-xs font-semibold text-indigo-400 hover:text-indigo-300 flex items-center gap-1 cursor-pointer transition-colors"
            >
              Open Enquiries Inbox <ExternalLink class="w-3 h-3" />
            </button>
          </div>
        </div>
      </Show>

      {/* Full Enquiries Scalable Inbox Modal */}
      <Modal
        isOpen={inboxOpen()}
        onClose={() => setInboxOpen(false)}
        title="Customer Inquiries & Leads Inbox"
        description="Search, filter, and review all prospective buyer inquiries in one unified console."
      >
        <div class="space-y-4">
          {/* Search & Filters Bar */}
          <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3 p-3 bg-slate-950 rounded-xl border border-slate-800">
            <div class="flex-1 relative">
              <Search class="w-4 h-4 text-slate-500 absolute left-3 top-2.5" />
              <input
                type="text"
                placeholder="Search by name, phone, email, or property..."
                value={inboxSearch()}
                onInput={(e) => {
                  setInboxSearch(e.currentTarget.value);
                  updateDebouncedInboxSearch(e.currentTarget.value);
                }}
                class="w-full bg-slate-900 border border-slate-800 rounded-lg pl-9 pr-3 py-1.5 text-xs text-slate-100 placeholder-slate-500 focus:outline-none focus:ring-1 focus:ring-indigo-500"
              />
            </div>

            <div class="flex items-center gap-2">
              <select
                value={inboxTypeFilter()}
                onChange={(e) => {
                  setInboxTypeFilter(e.currentTarget.value);
                  setInboxPage(1);
                }}
                class="bg-slate-900 border border-slate-800 rounded-lg px-2.5 py-1.5 text-xs text-slate-200 focus:outline-none"
              >
                <option value="all">All Types</option>
                <option value="project">Project Inquiries</option>
                <option value="property">Unit Inquiries</option>
              </select>

              <select
                value={inboxStatusFilter()}
                onChange={(e) => {
                  setInboxStatusFilter(e.currentTarget.value);
                  setInboxPage(1);
                }}
                class="bg-slate-900 border border-slate-800 rounded-lg px-2.5 py-1.5 text-xs text-slate-200 focus:outline-none"
              >
                <option value="all">All Status</option>
                <option value="unread">Unread Only</option>
                <option value="read">Read Only</option>
              </select>
            </div>
          </div>

          {/* Table of Inquiries */}
          <div class="border border-slate-800 rounded-xl overflow-hidden bg-slate-900 shadow-sm">
            <div class="overflow-x-auto max-h-[50vh]">
              <table class="w-full text-left text-xs">
                <thead class="bg-slate-950 text-slate-400 uppercase tracking-wider font-semibold border-b border-slate-800 sticky top-0 z-10">
                  <tr>
                    <th class="px-4 py-3">Lead / Buyer</th>
                    <th class="px-4 py-3">Inquired Entity</th>
                    <th class="px-4 py-3">Contact Details</th>
                    <th class="px-4 py-3">Inquiry Message</th>
                    <th class="px-4 py-3">Date</th>
                    <th class="px-4 py-3 text-right">Actions</th>
                  </tr>
                </thead>
                <tbody class="divide-y divide-slate-800/60">
                  <For each={paginatedInboxItems()}>
                    {(item) => (
                      <tr
                        class={`hover:bg-slate-850/50 transition-colors ${
                          !item.is_read ? "bg-indigo-950/20 font-medium" : ""
                        }`}
                      >
                        <td class="px-4 py-3 text-slate-200">
                          <div class="flex items-center gap-2">
                            <span
                              class={`w-2 h-2 rounded-full shrink-0 ${
                                !item.is_read ? "bg-indigo-500 animate-pulse" : "bg-transparent"
                              }`}
                            />
                            <span class="font-semibold">{item.name}</span>
                          </div>
                        </td>
                        <td class="px-4 py-3 text-slate-300">
                          <span
                            class={`px-1.5 py-0.5 text-[9px] uppercase font-bold mr-1.5 rounded ${
                              item.enquiry_type === "property"
                                ? "bg-emerald-950/70 text-emerald-400 border border-emerald-800/60"
                                : "bg-indigo-950/70 text-indigo-400 border border-indigo-800/60"
                            }`}
                          >
                            {item.enquiry_type}
                          </span>
                          {item.entity_name}
                        </td>
                        <td class="px-4 py-3 text-slate-300">
                          <div class="space-y-0.5">
                            <a
                              href={`tel:${item.phone}`}
                              class="text-slate-200 hover:text-indigo-400 flex items-center gap-1 font-mono"
                            >
                              <Phone class="w-3 h-3 text-slate-500" /> {item.phone}
                            </a>
                            <Show when={item.email}>
                              <a
                                href={`mailto:${item.email}`}
                                class="text-slate-400 hover:text-indigo-400 flex items-center gap-1 truncate max-w-[140px]"
                              >
                                <Mail class="w-3 h-3 text-slate-500" /> {item.email}
                              </a>
                            </Show>
                          </div>
                        </td>
                        <td class="px-4 py-3 text-slate-400 max-w-xs truncate">
                          {item.message ? `"${item.message}"` : <span class="italic text-slate-600">No message</span>}
                        </td>
                        <td class="px-4 py-3 text-slate-500 font-mono text-[11px] whitespace-nowrap">
                          {formatTimestamp(item.created_at)}
                        </td>
                        <td class="px-4 py-3 text-right">
                          <Show
                            when={!item.is_read}
                            fallback={
                              <span class="text-[11px] text-slate-500">Read</span>
                            }
                          >
                            <Button
                              size="sm"
                              variant="ghost"
                              onClick={() => markAsRead(item)}
                              class="text-indigo-400 hover:text-indigo-300 text-xs py-1"
                            >
                              <Check class="w-3.5 h-3.5" /> Mark read
                            </Button>
                          </Show>
                        </td>
                      </tr>
                    )}
                  </For>
                  {filteredInboxItems().length === 0 && (
                    <tr>
                      <td colspan="6" class="px-4 py-12 text-center text-slate-500">
                        No inquiries match your current search and filter criteria.
                      </td>
                    </tr>
                  )}
                </tbody>
              </table>
            </div>

            {/* Pagination Controls */}
            <div class="p-3 border-t border-slate-800 bg-slate-950 flex items-center justify-between text-xs text-slate-400">
              <span>
                Showing {filteredInboxItems().length === 0 ? 0 : (inboxPage() - 1) * PAGE_SIZE + 1} to{" "}
                {Math.min(inboxPage() * PAGE_SIZE, filteredInboxItems().length)} of{" "}
                {filteredInboxItems().length} inquiries
              </span>
              <div class="flex items-center gap-2">
                <Button
                  size="sm"
                  variant="ghost"
                  disabled={inboxPage() <= 1}
                  onClick={() => setInboxPage((p) => Math.max(1, p - 1))}
                >
                  Previous
                </Button>
                <span class="px-2 py-1 bg-slate-900 border border-slate-800 rounded font-mono text-slate-200">
                  {inboxPage()} / {totalInboxPages()}
                </span>
                <Button
                  size="sm"
                  variant="ghost"
                  disabled={inboxPage() >= totalInboxPages()}
                  onClick={() => setInboxPage((p) => Math.min(totalInboxPages(), p + 1))}
                >
                  Next
                </Button>
              </div>
            </div>
          </div>
        </div>
      </Modal>

      {/* Real-time Toast Alert */}
      <Show when={toast()}>
        {(t) => (
          <aside
            aria-label="New enquiry notification"
            class="fixed bottom-4 right-4 max-w-sm w-full bg-slate-900 border border-indigo-500/40 rounded-xl p-4 shadow-2xl shadow-black z-50 animate-in slide-in-from-bottom-5 duration-200"
          >
            <div class="flex items-start gap-3">
              <div class="p-2 bg-indigo-500/20 text-indigo-400 rounded-lg shrink-0">
                <Bell class="w-5 h-5 animate-bounce" />
              </div>
              <div class="flex-1 min-w-0">
                <p class="text-xs font-semibold text-white">{t().title}</p>
                <p class="text-xs text-slate-300 mt-0.5">{t().message}</p>
              </div>
              <button
                onClick={() => setToast(null)}
                class="text-slate-500 hover:text-white p-1 rounded-md"
              >
                <X class="w-4 h-4" />
              </button>
            </div>
          </aside>
        )}
      </Show>
    </div>
  );
}
