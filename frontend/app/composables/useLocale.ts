import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import dayjs from 'dayjs'
import 'dayjs/locale/en'
import 'dayjs/locale/pt-br'
import 'dayjs/locale/es'
import 'dayjs/locale/zh-cn'
import 'dayjs/locale/ru'


import { invoke } from '@tauri-apps/api/core'

export interface LocaleOption {
  code: 'en' | 'pt-BR' | 'es' | 'zh-CN' | 'ru'
  name: string
  label: string
  flag: string
  descriptionKey: string
}

export const SUPPORTED_LOCALES: LocaleOption[] = [
  {
    code: 'en',
    name: 'English',
    label: 'EN',
    flag: '🇺🇸',
    descriptionKey: 'settings.lang_en_desc'
  },
  {
    code: 'pt-BR',
    name: 'Português (Brasil)',
    label: 'PT',
    flag: '🇧🇷',
    descriptionKey: 'settings.lang_pt_desc'
  },
  {
    code: 'es',
    name: 'Español',
    label: 'ES',
    flag: '🇪🇸',
    descriptionKey: 'settings.lang_es_desc'
  },
  {
    code: 'zh-CN',
    name: '简体中文',
    label: 'ZH',
    flag: '🇨🇳',
    descriptionKey: 'settings.lang_zh_desc'
  },
  {
    code: 'ru',
    name: 'Русский',
    label: 'RU',
    flag: '🇷🇺',
    descriptionKey: 'settings.lang_ru_desc'
  }
]

const DAYJS_LOCALE_MAP: Record<string, string> = {
  'en': 'en',
  'pt-BR': 'pt-br',
  'es': 'es',
  'zh-CN': 'zh-cn',
  'ru': 'ru'
}

export function useAppLocale() {
  const { locale, t } = useI18n()

  const currentLocale = computed(() => locale.value)

  const setLocale = (newLocale: string) => {
    if (['en', 'pt-BR', 'es', 'zh-CN', 'ru'].includes(newLocale)) {
      locale.value = newLocale
      const dayjsCode = DAYJS_LOCALE_MAP[newLocale] || 'en'
      dayjs.locale(dayjsCode)
      if (typeof window !== 'undefined') {
        localStorage.setItem('atena_locale', newLocale)
      }
      invoke('update_tray_locale', { locale: newLocale }).catch(() => {})
    }
  }

  return {
    locale,
    currentLocale,
    setLocale,
    supportedLocales: SUPPORTED_LOCALES,
    t
  }
}
