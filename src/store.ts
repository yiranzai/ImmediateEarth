import { acceptHMRUpdate, defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import { readFile, readDir } from '@tauri-apps/plugin-fs'
import { join } from '@tauri-apps/api/path'

const versionString =
  import.meta.env.MODE === 'development' ? `${import.meta.env.VITE_APP_VERSION}-dev` : import.meta.env.VITE_APP_VERSION

function uint8ToBase64(bytes: Uint8Array): string {
  let binary = ''
  for (let i = 0; i < bytes.length; i++) {
    binary += String.fromCharCode(bytes[i])
  }
  return window.btoa(binary)
}

export const useStore = defineStore('main', {
  state: () => ({
    debug: import.meta.env.MODE === 'development',
    version: versionString,
    isInitialized: false,
    name: '',
    currentView: 'main' as 'main' | 'details',
    previewImage: '',
    mergedImagePath: '',
    latestImageLocalTime: '',
  }),

  actions: {
    initApp() {
      this.isInitialized = true
      console.log('app initialized!')
      this.findLatestImage()
    },
    setView(view: 'main' | 'details') {
      this.currentView = view
    },
    setPreviewData(data: { previewImage: string; mergedImagePath: string; latestImageLocalTime: string }) {
      this.previewImage = data.previewImage
      this.mergedImagePath = data.mergedImagePath
      this.latestImageLocalTime = data.latestImageLocalTime
    },
    async findLatestImage() {
      try {
        const dir = await invoke<string>('get_image_dir');
        const files = await readDir(dir);
        const imageFiles = files
          .filter(f => f.name && /^earth_\d{8}_\d{4}_black\.png$/.test(f.name))
          .sort((a, b) => (a.name! > b.name! ? 1 : -1));

        if (imageFiles.length > 0) {
          const latest = imageFiles[imageFiles.length - 1];
          const imagePath = await join(dir, latest.name!);
          const imageName = latest.name!;

          const imageBytes = await readFile(imagePath);
          const base64Data = uint8ToBase64(imageBytes);

          const match = imageName.match(/earth_(\d{8})_(\d{4})/);
          let localTime = '';
          if (match) {
            const [, dateStr, timeStr] = match;
            const utcTime = new Date(
              `${dateStr.substring(0, 4)}-${dateStr.substring(4, 6)}-${dateStr.substring(6, 8)}T${timeStr.substring(0, 2)}:${timeStr.substring(2, 4)}:00Z`
            );
            localTime = utcTime.toLocaleString();
          }

          this.setPreviewData({
            previewImage: `data:image/png;base64,${base64Data}`,
            mergedImagePath: imagePath,
            latestImageLocalTime: localTime,
          });
        }
      } catch (e) {
        console.error('在 store 中查找最新图片失败', e);
      }
    }
  },

  getters: {
    isReady: (state) => {
      return !state.isInitialized
    },

    storeGreet: (state) => {
      if (state.name.length > 0) {
        return `Greetings from Pinia store, ${state.name}!`
      }
      return ''
    },
  },
})

if (import.meta.hot) {
  import.meta.hot.accept(acceptHMRUpdate(useStore, import.meta.hot))
}
