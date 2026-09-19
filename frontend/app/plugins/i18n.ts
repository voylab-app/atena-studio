import { createI18n } from 'vue-i18n'
import ptBR from '~/locales/pt-BR.json'
import en from '~/locales/en.json'
import es from '~/locales/es.json'
import zhCN from '~/locales/zh-CN.json'
import ru from '~/locales/ru.json'

export default defineNuxtPlugin((nuxtApp) => {
  let initialLocale = 'pt-BR'

  if (typeof window !== 'undefined') {
    const saved = localStorage.getItem('atena_locale')
    if (saved && ['pt-BR', 'en', 'es', 'zh-CN', 'ru'].includes(saved)) {
      initialLocale = saved
    } else {
      const navLang = (navigator.language || '').toLowerCase()
      if (navLang.startsWith('en')) {
        initialLocale = 'en'
      } else if (navLang.startsWith('es')) {
        initialLocale = 'es'
      } else if (navLang.startsWith('zh')) {
        initialLocale = 'zh-CN'
      } else if (navLang.startsWith('ru')) {
        initialLocale = 'ru'
      } else {
        initialLocale = 'pt-BR'
      }
    }
  }

  const i18n = createI18n({
    legacy: false,
    locale: initialLocale,
    fallbackLocale: 'en',
    messages: {
      'pt-BR': ptBR,
      en,
      es,
      'zh-CN': zhCN,
      ru
    }
  })

  nuxtApp.vueApp.use(i18n)

  if (typeof window !== 'undefined') {
    import('@tauri-apps/api/core')
      .then(({ invoke }) => {
        invoke('update_tray_locale', { locale: initialLocale }).catch(() => {})
      })
      .catch(() => {})
  }

  return {
    provide: {
      i18n
    }
  }
})
