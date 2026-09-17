import { ref, type Ref } from 'vue'

export const isSpeaking: Ref<boolean> = ref(false)
export const isPaused: Ref<boolean> = ref(false)
export const activeSpeakingId: Ref<string | null> = ref(null)

let currentUtterance: SpeechSynthesisUtterance | null = null

/**
 * Strips markdown, code blocks and tags to create clean plain speech text
 */
export function cleanTextForSpeech(text?: string | null): string {
  if (!text) return ''
  let t = text
  // Remove markdown code blocks
  t = t.replace(/```[\s\S]*?```/g, ' [Bloco de código omitido] ')
  // Remove inline code
  t = t.replace(/`([^`]+)`/g, '$1')
  // Remove URLs
  t = t.replace(/https?:\/\/\S+/g, 'link')
  // Remove headers
  t = t.replace(/^#{1,6}\s+/gm, '')
  // Remove bold / italic markers
  t = t.replace(/[*_]{1,3}([^*_]+)[*_]{1,3}/g, '$1')
  // Remove image / link markdown syntax
  t = t.replace(/!\[([^\]]*)\]\([^)]*\)/g, '$1')
  t = t.replace(/\[([^\]]*)\]\([^)]*\)/g, '$1')
  // Remove HTML tags
  t = t.replace(/<[^>]+>/g, '')
  // Normalize whitespace
  t = t.replace(/\s+/g, ' ').trim()
  return t
}

/**
 * Gets best voice for pt-BR or fallback
 */
function getBestVoice(): SpeechSynthesisVoice | null {
  if (typeof window === 'undefined' || !window.speechSynthesis) return null
  const voices = window.speechSynthesis.getVoices()
  
  // Try preferred Portuguese voices (macOS / Chrome / Edge)
  const ptVoices = voices.filter(
    (v) => v.lang === 'pt-BR' || v.lang === 'pt_BR' || v.lang.startsWith('pt')
  )

  if (ptVoices.length > 0) {
    // Prefer high quality macOS voices if present
    const preferred = ptVoices.find(
      (v) =>
        v.name.includes('Luciana') ||
        v.name.includes('Felipe') ||
        v.name.includes('Daniel') ||
        v.name.includes('Premium') ||
        v.name.includes('Enhanced')
    )
    return preferred || ptVoices[0] || null
  }

  // Fallback to default system voice
  return voices.find((v) => v.default) || voices[0] || null
}

/**
 * Start or Toggle TTS speaking
 */
export function speakText(id: string, rawText?: string | null): void {
  if (typeof window === 'undefined' || !window.speechSynthesis) {
    console.warn('Speech Synthesis API não suportada neste ambiente.')
    return
  }

  // If already speaking this specific message, toggle pause/play or stop
  if (activeSpeakingId.value === id) {
    if (window.speechSynthesis.speaking) {
      if (window.speechSynthesis.paused) {
        window.speechSynthesis.resume()
        isPaused.value = false
        isSpeaking.value = true
        return
      } else {
        window.speechSynthesis.pause()
        isPaused.value = true
        isSpeaking.value = false
        return
      }
    }
  }

  // Stop any previous speech
  stopSpeech()

  const cleanText = cleanTextForSpeech(rawText)
  if (!cleanText) return

  const utterance = new SpeechSynthesisUtterance(cleanText)
  const voice = getBestVoice()
  if (voice) {
    utterance.voice = voice
    utterance.lang = voice.lang
  } else {
    utterance.lang = 'pt-BR'
  }

  utterance.rate = 1.05 // Slightly faster for natural assistant feel
  utterance.pitch = 1.0

  utterance.onstart = () => {
    activeSpeakingId.value = id
    isSpeaking.value = true
    isPaused.value = false
  }

  utterance.onend = () => {
    if (activeSpeakingId.value === id) {
      activeSpeakingId.value = null
      isSpeaking.value = false
      isPaused.value = false
    }
  }

  utterance.onerror = (e) => {
    console.warn('TTS erro:', e)
    if (activeSpeakingId.value === id) {
      activeSpeakingId.value = null
      isSpeaking.value = false
      isPaused.value = false
    }
  }

  currentUtterance = utterance
  window.speechSynthesis.speak(utterance)
}

/**
 * Stop any ongoing TTS
 */
export function stopSpeech(): void {
  if (typeof window !== 'undefined' && window.speechSynthesis) {
    window.speechSynthesis.cancel()
  }
  activeSpeakingId.value = null
  isSpeaking.value = false
  isPaused.value = false
  currentUtterance = null
}
