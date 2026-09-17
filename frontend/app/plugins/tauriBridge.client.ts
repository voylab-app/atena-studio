import { initIpcBridge } from '~/utils/ipcAdapter'

export default defineNuxtPlugin({
  name: 'tauri-bridge-client',
  enforce: 'pre',
  setup() {
    initIpcBridge()
  }
})
