import { useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { readFile } from '@tauri-apps/plugin-fs';
import { useStore } from '@/store';
import type { MattingTask, MattingSettings, MattingResult } from '@/types';

function generateId() {
  return Math.random().toString(36).substring(2, 15);
}

export function useTauri() {
  const addTasks = useStore((s) => s.addTasks);
  const updateTask = useStore((s) => s.updateTask);
  const updateTaskResult = useStore((s) => s.updateTaskResult);
  const setProcessing = useStore((s) => s.setProcessing);
  const currentSettings = useStore((s) => s.currentSettings);
  const appSettings = useStore((s) => s.appSettings);

  const selectFiles = useCallback(async () => {
    const selected = await open({
      multiple: true,
      filters: [
        { name: '图片文件', extensions: ['png', 'jpg', 'jpeg', 'webp', 'bmp'] },
        { name: '所有文件', extensions: ['*'] },
      ],
    });
    if (!selected) return;
    const paths = Array.isArray(selected) ? selected : [selected];

    const tasks: MattingTask[] = [];
    for (const path of paths) {
      try {
        const data = await readFile(path);
        const blob = new Blob([data]);
        const url = URL.createObjectURL(blob);
        const img = await new Promise<HTMLImageElement>((resolve, reject) => {
          const i = new Image();
          i.onload = () => resolve(i);
          i.onerror = reject;
          i.src = url;
        });

        const canvas = document.createElement('canvas');
        canvas.width = 160;
        canvas.height = Math.round((160 / img.width) * img.height);
        const ctx = canvas.getContext('2d')!;
        ctx.drawImage(img, 0, 0, canvas.width, canvas.height);

        const task: MattingTask = {
          id: generateId(),
          fileName: path.split(/[\\/]/).pop() ?? 'unknown',
          filePath: path,
          thumbnail: canvas.toDataURL('image/jpeg', 0.6),
          status: 'idle',
          progress: 0,
          settings: { ...currentSettings },
        };

        tasks.push(task);
        URL.revokeObjectURL(url);
      } catch (e) {
        console.error('读取文件失败:', path, e);
      }
    }

    addTasks(tasks);
  }, [addTasks, currentSettings]);

  const selectOutputFolder = useCallback(async () => {
    const folder = await open({ directory: true });
    if (folder && typeof folder === 'string') {
      useStore.getState().updateAppSettings({ outputDir: folder });
    }
    return folder;
  }, []);

  const startProcessing = useCallback(async (taskIds?: string[]) => {
    const state = useStore.getState();
    const tasks = state.tasks;
    const pending = taskIds
      ? tasks.filter((t) => taskIds.includes(t.id) && t.status === 'idle')
      : tasks.filter((t) => t.status === 'idle');
    if (pending.length === 0) return;

    setProcessing(true);
    const total = pending.length;

    for (let i = 0; i < pending.length; i++) {
      const task = pending[i];
      updateTask(task.id, { status: 'processing', progress: 10 });

      try {
        // 1. 抠图处理
        const resultPath: string = await invoke('process_image', {
          params: {
            filePath: task.filePath,
            settings: task.settings,
          },
        });

        updateTask(task.id, { status: 'processing', progress: 80 });

        // 2. 获取结果信息
        const fileInfo: { width: number; height: number; size: number } = await invoke('get_file_info', {
          path: resultPath,
        });

        const result: MattingResult = {
          outputPath: resultPath,
          width: fileInfo.width,
          height: fileInfo.height,
          format: task.settings.outputFormat,
          fileSize: fileInfo.size,
        };

        // 3. 生成预览
        const resultData = await readFile(resultPath);
        const blob = new Blob([resultData]);
        const url = URL.createObjectURL(blob);
        const resultImg = await new Promise<HTMLImageElement>((resolve, reject) => {
          const image = new Image();
          image.onload = () => resolve(image);
          image.onerror = reject;
          image.src = url;
        });

        const previewCanvas = document.createElement('canvas');
        previewCanvas.width = 200;
        previewCanvas.height = Math.round((200 / resultImg.width) * resultImg.height);
        const pCtx = previewCanvas.getContext('2d')!;
        pCtx.drawImage(resultImg, 0, 0, previewCanvas.width, previewCanvas.height);

        const previewPath = previewCanvas.toDataURL('image/png');
        URL.revokeObjectURL(url);

        // 4. 保存结果
        updateTaskResult(task.id, { ...result, previewPath });

        updateTask(task.id, { progress: 100 });
      } catch (e: any) {
        updateTask(task.id, {
          status: 'error',
          error: String(e),
        });
      }

      useStore.setState({ globalProgress: Math.round(((i + 1) / total) * 100) });
    }

    setProcessing(false);
  }, [setProcessing, updateTask, updateTaskResult, appSettings]);

  const saveImage = useCallback(async (taskId: string) => {
    const state = useStore.getState();
    const task = state.tasks.find((t) => t.id === taskId);
    if (!task || !task.result) return;

    await invoke('save_image', {
      resultPath: task.result.outputPath,
      settings: {
        ...task.settings,
        fileName: task.fileName,
      },
      outputDir: state.appSettings.outputDir,
    });
  }, []);

  const saveAll = useCallback(async () => {
    const state = useStore.getState();
    const completed = state.tasks.filter((t) => t.status === 'completed' && t.result);
    for (const task of completed) {
      await invoke('save_image', {
        resultPath: task.result!.outputPath,
        settings: {
          ...task.settings,
          fileName: task.fileName,
        },
        outputDir: state.appSettings.outputDir,
      });
    }
  }, []);

  const openInFolder = useCallback(async (taskId: string) => {
    const task = useStore.getState().tasks.find((t) => t.id === taskId);
    if (!task || !task.result) return;
    await invoke('open_in_folder', { path: task.result.outputPath });
  }, []);

  return {
    selectFiles,
    selectOutputFolder,
    startProcessing,
    saveImage,
    saveAll,
    openInFolder,
  };
}
