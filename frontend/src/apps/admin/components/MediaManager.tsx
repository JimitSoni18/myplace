import { createSignal, For, Show } from "solid-js";
import {
  Upload,
  Trash2,
  ArrowUp,
  ArrowDown,
  FileText,
  GripVertical,
  Star,
  Eye,
  X,
} from "lucide-solid";
import { api } from "@/shared/api/client";
import type { ReorderMediaPayload } from "@/shared/api/types";

export interface MediaItem {
  id: string;
  media_type: string;
  url: string;
  thumbnail_url?: string;
  sequence: number;
}

interface MediaManagerProps {
  uploadEndpoint: string;
  reorderEndpoint?: string;
  deleteEndpoint: (mediaId: string) => string;
  media: MediaItem[];
  onRefresh: () => void;
  accept?: string;
  title?: string;
}

export function MediaManager(props: MediaManagerProps) {
  const [uploading, setUploading] = createSignal(false);
  const [zoneDragOver, setZoneDragOver] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  // Drag and drop state for items
  const [draggedIndex, setDraggedIndex] = createSignal<number | null>(null);
  const [dropTargetIndex, setDropTargetIndex] = createSignal<number | null>(null);
  const [reordering, setReordering] = createSignal(false);

  // Lightbox preview state
  const [previewMedia, setPreviewMedia] = createSignal<{ url: string; type: string } | null>(null);

  const handleUploadFiles = async (files: FileList | null) => {
    if (!files || files.length === 0) return;
    setError(null);

    const MAX_IMAGE_SIZE = 5 * 1024 * 1024; // 5 MB
    const validFiles: File[] = [];
    for (let i = 0; i < files.length; i++) {
      const file = files[i];
      if (file.type === "image/svg+xml" || file.name.toLowerCase().endsWith(".svg")) {
        setError("SVG format is not supported for media gallery. Please use JPEG, PNG, or WebP.");
        return;
      }
      if (file.type.startsWith("image/") && file.size > MAX_IMAGE_SIZE) {
        setError(`"${file.name}" exceeds the 5MB image limit (${(file.size / (1024 * 1024)).toFixed(2)} MB).`);
        return;
      }
      validFiles.push(file);
    }

    setUploading(true);

    const formData = new FormData();
    for (const file of validFiles) {
      formData.append("files", file);
    }

    try {
      await api.upload(props.uploadEndpoint, formData);
      props.onRefresh();
    } catch (e: any) {
      setError(e.message || "Failed to upload file(s)");
    } finally {
      setUploading(false);
    }
  };


  const handleDelete = async (id: string) => {
    if (!confirm("Are you sure you want to delete this media item?")) return;
    try {
      await api.delete(props.deleteEndpoint(id));
      props.onRefresh();
    } catch (e: any) {
      alert(e.message || "Failed to delete media item");
    }
  };

  const commitReorder = async (newItems: MediaItem[]) => {
    if (!props.reorderEndpoint) return;
    setReordering(true);
    setError(null);
    try {
      const payload: ReorderMediaPayload = {
        order: newItems.map((item) => item.id),
      };
      await api.post(props.reorderEndpoint, payload);
      props.onRefresh();
    } catch (e: any) {
      setError(e.message || "Failed to persist media sequence order");
      props.onRefresh();
    } finally {
      setReordering(false);
    }
  };

  const handleDragStart = (e: DragEvent, index: number) => {
    e.dataTransfer?.setData("text/plain", index.toString());
    e.dataTransfer!.effectAllowed = "move";
    setDraggedIndex(index);
  };

  const handleDragOver = (e: DragEvent, index: number) => {
    e.preventDefault();
    if (draggedIndex() !== null && draggedIndex() !== index) {
      setDropTargetIndex(index);
    }
  };

  const handleDrop = async (e: DragEvent, targetIdx: number) => {
    e.preventDefault();
    const sourceIdx = draggedIndex();
    setDraggedIndex(null);
    setDropTargetIndex(null);

    if (sourceIdx === null || sourceIdx === targetIdx) return;

    const newItems = [...props.media];
    const [moved] = newItems.splice(sourceIdx, 1);
    newItems.splice(targetIdx, 0, moved);

    await commitReorder(newItems);
  };

  const moveItem = async (index: number, direction: -1 | 1) => {
    const targetIdx = index + direction;
    if (targetIdx < 0 || targetIdx >= props.media.length) return;

    const newItems = [...props.media];
    const [moved] = newItems.splice(index, 1);
    newItems.splice(targetIdx, 0, moved);

    await commitReorder(newItems);
  };

  return (
    <div class="space-y-4">
      {/* Upload Drop Zone */}
      <div
        onDragOver={(e) => {
          e.preventDefault();
          setZoneDragOver(true);
        }}
        onDragLeave={() => setZoneDragOver(false)}
        onDrop={(e) => {
          e.preventDefault();
          setZoneDragOver(false);
          handleUploadFiles(e.dataTransfer?.files || null);
        }}
        class={`border-2 border-dashed rounded-xl p-8 text-center transition-all cursor-pointer ${
          zoneDragOver()
            ? "border-indigo-500 bg-indigo-950/30 ring-4 ring-indigo-500/20"
            : "border-slate-800 hover:border-slate-700 bg-slate-900/40 hover:bg-slate-900/60"
        }`}
        onClick={() => {
          const input = document.createElement("input");
          input.type = "file";
          input.multiple = true;
          input.accept = props.accept || "image/jpeg,image/png,image/webp,video/mp4,video/quicktime,video/webm";
          input.onchange = (e) => {
            const target = e.target as HTMLInputElement;
            handleUploadFiles(target.files);
          };
          input.click();
        }}
      >
        <Upload class="w-8 h-8 mx-auto text-indigo-400 mb-2 transition-transform duration-200 group-hover:-translate-y-0.5" />
        <p class="text-sm font-semibold text-slate-100">
          {uploading() ? "Processing and uploading to storage..." : "Click or drag & drop marketing media here"}
        </p>
        <p class="text-xs text-slate-400 mt-1 max-w-md mx-auto">
          High-resolution renders, photos, or walkthrough videos. Server validates magic bytes and auto-generates WebP thumbnails.
        </p>
      </div>

      {error() && (
        <div class="p-3 bg-rose-950/50 border border-rose-800 rounded-lg text-xs text-rose-300 flex items-center justify-between">
          <span>{error()}</span>
          <button onClick={() => setError(null)} class="text-rose-400 hover:text-rose-200">
            <X class="w-3.5 h-3.5" />
          </button>
        </div>
      )}

      {/* Media Guidelines & Drag-and-Drop Cue */}
      {props.media.length > 0 && (
        <div class="flex items-center justify-between text-xs text-slate-400 px-1">
          <div class="flex items-center gap-1.5">
            <GripVertical class="w-3.5 h-3.5 text-indigo-400" />
            <span>Drag cards to reorder gallery. The first item is the primary cover image.</span>
          </div>
          {reordering() && <span class="text-indigo-400 font-mono animate-pulse">Saving order...</span>}
        </div>
      )}

      {/* Grid of uploaded media with HTML5 Drag-and-Drop */}
      <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-4">
        <For each={props.media}>
          {(item, index) => {
            const isImage =
              item.media_type === "IMAGE" ||
              item.media_type === "image" ||
              item.url.match(/\.(webp|jpg|jpeg|png)$/i);
            const isCover = index() === 0;
            const isBeingDragged = draggedIndex() === index();
            const isTarget = dropTargetIndex() === index();

            return (
              <div
                draggable={Boolean(props.reorderEndpoint)}
                onDragStart={(e) => handleDragStart(e, index())}
                onDragOver={(e) => handleDragOver(e, index())}
                onDrop={(e) => handleDrop(e, index())}
                onDragEnd={() => {
                  setDraggedIndex(null);
                  setDropTargetIndex(null);
                }}
                class={`group relative bg-slate-900 border rounded-xl overflow-hidden shadow-sm flex flex-col transition-all duration-200 ${
                  isBeingDragged
                    ? "opacity-30 scale-95 border-indigo-500 shadow-2xl"
                    : isTarget
                    ? "border-indigo-400 ring-2 ring-indigo-500/50 scale-102"
                    : "border-slate-800 hover:border-slate-700"
                } ${props.reorderEndpoint ? "cursor-grab active:cursor-grabbing" : ""}`}
              >
                {/* Visual Thumbnail / Preview Box */}
                <div class="aspect-video bg-slate-950 flex items-center justify-center overflow-hidden relative">
                  {isImage ? (
                    <img
                      src={item.thumbnail_url || item.url}
                      alt="Media item"
                      class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-300"
                    />
                  ) : (
                    <div class="flex flex-col items-center justify-center text-slate-500">
                      <FileText class="w-8 h-8 mb-1" />
                      <span class="text-[10px] uppercase font-mono">{item.media_type}</span>
                    </div>
                  )}

                  {/* Top Badges */}
                  <div class="absolute top-2 left-2 right-2 flex items-center justify-between pointer-events-none">
                    {isCover ? (
                      <span class="px-2 py-0.5 rounded-md bg-indigo-600 text-[10px] font-bold text-white shadow-md flex items-center gap-1">
                        <Star class="w-3 h-3 fill-current text-amber-300" /> Cover Photo
                      </span>
                    ) : (
                      <span class="px-1.5 py-0.5 rounded bg-slate-950/80 text-[10px] font-mono text-slate-300 backdrop-blur-sm border border-slate-800">
                        #{index() + 1}
                      </span>
                    )}

                    <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity pointer-events-auto">
                      <button
                        onClick={() => setPreviewMedia({ url: item.url, type: item.media_type })}
                        title="View Full Resolution"
                        class="p-1 rounded bg-slate-900/90 text-slate-300 hover:text-white backdrop-blur-sm border border-slate-700 hover:bg-slate-800"
                      >
                        <Eye class="w-3 h-3" />
                      </button>
                    </div>
                  </div>
                </div>

                {/* Bottom Bar: Accessible Controls & Delete */}
                <div class="p-2 flex items-center justify-between bg-slate-900/90 border-t border-slate-800/80">
                  <div class="flex items-center gap-1 text-slate-400">
                    <GripVertical class="w-3.5 h-3.5 text-slate-500" />
                    {props.reorderEndpoint && (
                      <>
                        <button
                          disabled={index() === 0 || reordering()}
                          onClick={() => moveItem(index(), -1)}
                          title="Move up"
                          class="p-1 text-slate-400 hover:text-white disabled:opacity-30 rounded hover:bg-slate-800 transition-colors"
                        >
                          <ArrowUp class="w-3 h-3" />
                        </button>
                        <button
                          disabled={index() === props.media.length - 1 || reordering()}
                          onClick={() => moveItem(index(), 1)}
                          title="Move down"
                          class="p-1 text-slate-400 hover:text-white disabled:opacity-30 rounded hover:bg-slate-800 transition-colors"
                        >
                          <ArrowDown class="w-3 h-3" />
                        </button>
                      </>
                    )}
                  </div>

                  <button
                    onClick={() => handleDelete(item.id)}
                    title="Delete media"
                    class="p-1 text-slate-400 hover:text-rose-400 rounded hover:bg-slate-800 transition-colors"
                  >
                    <Trash2 class="w-3.5 h-3.5" />
                  </button>
                </div>
              </div>
            );
          }}
        </For>
      </div>

      {props.media.length === 0 && (
        <div class="text-center py-8 text-xs text-slate-500 border border-slate-850 rounded-xl bg-slate-900/20">
          No visual media uploaded yet for this resource. Upload renders, photos, or videos above.
        </div>
      )}

      {/* Lightbox Preview Modal */}
      <Show when={previewMedia()}>
        {(media) => (
          <div
            class="fixed inset-0 z-50 bg-slate-950/90 backdrop-blur-md flex items-center justify-center p-4"
            onClick={() => setPreviewMedia(null)}
          >
            <div
              class="relative max-w-5xl max-h-[90vh] bg-slate-900 rounded-2xl overflow-hidden border border-slate-800 shadow-2xl flex flex-col"
              onClick={(e) => e.stopPropagation()}
            >
              <div class="flex items-center justify-between px-4 py-3 border-b border-slate-800 bg-slate-950">
                <span class="text-xs font-semibold text-slate-300">Media Preview</span>
                <button
                  onClick={() => setPreviewMedia(null)}
                  class="p-1 text-slate-400 hover:text-white rounded-lg hover:bg-slate-800"
                >
                  <X class="w-4 h-4" />
                </button>
              </div>
              <div class="p-2 overflow-auto flex items-center justify-center">
                <img
                  src={media().url}
                  alt="High resolution preview"
                  class="max-h-[80vh] w-auto object-contain rounded-lg"
                />
              </div>
            </div>
          </div>
        )}
      </Show>
    </div>
  );
}
