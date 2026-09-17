// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  ssr: false,
  future: { compatibilityVersion: 4 },
  compatibilityDate: '2025-07-15',
  devtools: { enabled: false },
  typescript: {
    strict: true,
    typeCheck: true
  },
  modules: [
    '@nuxtjs/tailwindcss'
  ],
  css: [
    'highlight.js/styles/atom-one-dark.css',
    '~/assets/css/main.css'
  ],
  vite: {
    clearScreen: false,
    server: {
      strictPort: true,
    }
  }
})
