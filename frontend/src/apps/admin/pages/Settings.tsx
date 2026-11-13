import { createSignal } from "solid-js";
import { Sparkles, ShieldAlert, Sliders } from "lucide-solid";
import { Amenities } from "./Amenities";
import { AdminUsers } from "./AdminUsers";

export function Settings() {
  const [activeTab, setActiveTab] = createSignal<"amenities" | "users">("amenities");

  return (
    <div class="space-y-6">
      {/* Settings Header */}
      <div>
        <h1 class="text-2xl font-bold tracking-tight text-white flex items-center gap-2">
          <Sliders class="w-6 h-6 text-indigo-400" />
          Platform Settings & Taxonomy
        </h1>
        <p class="text-xs text-slate-400 mt-1">
          Configure system-wide master data and administrative access controls.
        </p>
      </div>

      {/* Navigation Tabs */}
      <div class="border-b border-slate-800 flex items-center gap-2 overflow-x-auto">
        <button
          onClick={() => setActiveTab("amenities")}
          class={`flex items-center gap-2 px-4 py-2.5 text-xs font-semibold border-b-2 transition-all cursor-pointer shrink-0 ${
            activeTab() === "amenities"
              ? "border-indigo-500 text-indigo-400 bg-indigo-500/10 rounded-t-lg"
              : "border-transparent text-slate-400 hover:text-slate-200 hover:border-slate-700"
          }`}
        >
          <Sparkles class="w-3.5 h-3.5" />
          Amenities Master List
        </button>

        <button
          onClick={() => setActiveTab("users")}
          class={`flex items-center gap-2 px-4 py-2.5 text-xs font-semibold border-b-2 transition-all cursor-pointer shrink-0 ${
            activeTab() === "users"
              ? "border-indigo-500 text-indigo-400 bg-indigo-500/10 rounded-t-lg"
              : "border-transparent text-slate-400 hover:text-slate-200 hover:border-slate-700"
          }`}
        >
          <ShieldAlert class="w-3.5 h-3.5" />
          Administrative Staff
        </button>
      </div>

      {/* Tab Panels */}
      <div>
        {activeTab() === "amenities" && <Amenities />}
        {activeTab() === "users" && <AdminUsers />}
      </div>
    </div>
  );
}
