import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { MattingTask, MattingSettings, AppSettings, ModelStatus, ModelInfo, ModelParams } from "@/types";
import { DEFAULT_APP_SETTINGS } from "@/types";

interface AppState {
  tasks: MattingTask[];
  selectedTaskId: string | null;
  currentSettings: MattingSettings;
  appSettings: AppSettings;
  isProcessing: boolean;
  globalProgress: number;
  dragOver: boolean;
  modelStatus: ModelStatus;
  modelDialogOpen: boolean;
  konvaExportFn: ((mimeType?: string, quality?: number) => string | null) | null;
  availableModels: ModelInfo[];
  activeModelId: string;
  modelParams: ModelParams;
  modelSwitching: boolean;
  modelSwitchingError: string | null;

  // Actions
  addTasks: (tasks: MattingTask[]) => void;
  removeTask: (id: string) => void;
  selectTask: (id: string | null) => void;
  updateTask: (id: string, updates: Partial<MattingTask>) => void;
  updateTaskResult: (id: string, result: MattingTask["result"]) => void;
  updateSettings: (settings: Partial<MattingSettings>) => void;
  updateAppSettings: (settings: Partial<AppSettings>) => void;
  setProcessing: (processing: boolean) => void;
  setGlobalProgress: (progress: number) => void;
  setDragOver: (over: boolean) => void;
  clearCompleted: () => void;
  clearAll: () => void;
  setModelStatus: (status: Partial<ModelStatus>) => void;
  setModelDialogOpen: (open: boolean) => void;
  setKonvaExportFn: (fn: ((mimeType?: string, quality?: number) => string | null) | null) => void;
  setAvailableModels: (models: ModelInfo[]) => void;
  setActiveModelId: (id: string) => void;
  setModelParams: (params: ModelParams) => void;
  setModelSwitching: (switching: boolean) => void;
  setModelSwitchingError: (error: string | null) => void;
}

export const useStore = create<AppState>()(
  persist(
    (set, get) => ({
      tasks: [],
      selectedTaskId: null,
      currentSettings: { ...DEFAULT_APP_SETTINGS.defaultSettings },
      appSettings: { ...DEFAULT_APP_SETTINGS },
      isProcessing: false,
      globalProgress: 0,
      dragOver: false,
      modelStatus: {
        exists: false,
        path: "",
        size: 0,
        downloading: false,
        progress: 0,
        bytesDownloaded: 0,
        totalBytes: 0,
        speed: 0,
        state: "notDownloaded" as const,
      },
      modelDialogOpen: false,
      konvaExportFn: null,
      availableModels: [],
      activeModelId: "birefnet",
      modelParams: {},
      modelSwitching: false,
      modelSwitchingError: null,

      addTasks: (newTasks) =>
        set((state) => {
          const existingIds = new Set(state.tasks.map((t) => t.id));
          const filtered = newTasks.filter((t) => !existingIds.has(t.id));
          return {
            tasks: [...state.tasks, ...filtered],
            selectedTaskId: state.selectedTaskId || filtered[0]?.id || null,
          };
        }),

      removeTask: (id) =>
        set((state) => {
          const tasks = state.tasks.filter((t) => t.id !== id);
          return {
            tasks,
            selectedTaskId:
              state.selectedTaskId === id
                ? tasks[tasks.length - 1]?.id || null
                : state.selectedTaskId,
          };
        }),

      selectTask: (id) => set({ selectedTaskId: id }),

      updateTask: (id, updates) =>
        set((state) => ({
          tasks: state.tasks.map((t) => (t.id === id ? { ...t, ...updates } : t)),
        })),

      updateTaskResult: (id, result) =>
        set((state) => ({
          tasks: state.tasks.map((t) =>
            t.id === id ? { ...t, result, status: "completed" as const, progress: 100 } : t
          ),
        })),

      updateSettings: (settings) =>
        set((state) => ({
          currentSettings: { ...state.currentSettings, ...settings },
        })),

      updateAppSettings: (settings) =>
        set((state) => ({
          appSettings: { ...state.appSettings, ...settings },
        })),

      setProcessing: (processing) => set({ isProcessing: processing }),

      setGlobalProgress: (progress) => set({ globalProgress: progress }),

      setDragOver: (over) => set({ dragOver: over }),

      clearCompleted: () =>
        set((state) => {
          const remaining = state.tasks.filter((t) => t.status !== "completed");
          return {
            tasks: remaining,
            selectedTaskId:
              remaining.find((t) => t.id === state.selectedTaskId)?.id ||
              remaining[remaining.length - 1]?.id ||
              null,
          };
        }),

      clearAll: () => set({ tasks: [], selectedTaskId: null }),

      setModelStatus: (status) =>
        set((state) => ({
          modelStatus: { ...state.modelStatus, ...status },
        })),

      setModelDialogOpen: (open) => set({ modelDialogOpen: open }),

      setKonvaExportFn: (fn) => set({ konvaExportFn: fn }),

      setAvailableModels: (models) => set({ availableModels: models }),

      setActiveModelId: (id) => set({ activeModelId: id }),

      setModelParams: (params) => set({ modelParams: params }),

      setModelSwitching: (switching) => set({ modelSwitching: switching }),

      setModelSwitchingError: (error) => set({ modelSwitchingError: error }),
    }),
    {
      name: "mopng-desktop-store",
      partialize: (state) => ({
        appSettings: state.appSettings,
        currentSettings: state.currentSettings,
        activeModelId: state.activeModelId,
        modelParams: state.modelParams,
      }),
    }
  )
);
