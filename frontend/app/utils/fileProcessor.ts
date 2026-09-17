// WebKit / Safari polyfills required by PDF.js (v4+)
if (typeof window !== 'undefined') {
  if (typeof ReadableStream !== 'undefined' && !(ReadableStream.prototype as any)[Symbol.asyncIterator]) {
    (ReadableStream.prototype as any)[Symbol.asyncIterator] = function () {
      const reader = this.getReader()
      return {
        next() {
          return reader.read().then(({ done, value }: { done: boolean; value: any }) => {
            if (done) {
              reader.releaseLock()
              return { done: true, value: undefined }
            }
            return { done: false, value }
          })
        },
        return() {
          reader.releaseLock()
          return Promise.resolve({ done: true, value: undefined })
        },
        [Symbol.asyncIterator]() {
          return this
        }
      }
    }
  }

  if (typeof (Promise as any).withResolvers === 'undefined') {
    ;(Promise as any).withResolvers = function () {
      let resolve: any, reject: any
      const promise = new Promise((res, rej) => {
        resolve = res
        reject = rej
      })
      return { promise, resolve, reject }
    }
  }
}

import * as pdfjsLib from 'pdfjs-dist/build/pdf.min.mjs'
// @ts-ignore
import pdfWorkerUrl from 'pdfjs-dist/build/pdf.worker.min.mjs?url'

// Configure PDF.js worker locally for Vite/Tauri offline support
if (typeof window !== 'undefined') {
  try {
    pdfjsLib.GlobalWorkerOptions.workerSrc = pdfWorkerUrl
  } catch (e) {
    console.warn('PDF.js worker setup fallback:', e)
  }
}

// Lazy-loaded Whisper pipeline for audio transcription
let whisperPipeline: any = null

export const VIDEO_EXTENSIONS = ['.mp4', '.mov', '.avi', '.mkv', '.webm', '.flv', '.wmv', '.m4v', '.3gp']
export const AUDIO_EXTENSIONS = ['.mp3', '.wav', '.m4a', '.ogg', '.aac', '.flac', '.wma', '.opus', '.oga']
export const IMAGE_EXTENSIONS = ['.png', '.jpg', '.jpeg', '.webp', '.gif', '.svg', '.bmp', '.ico']
export const CODE_TEXT_EXTENSIONS = [
  '.txt', '.md', '.markdown', '.json', '.csv', '.tsv', '.py', '.js', '.jsx', '.ts', '.tsx',
  '.rs', '.go', '.c', '.cpp', '.h', '.hpp', '.java', '.kt', '.swift', '.rb', '.php',
  '.html', '.htm', '.css', '.scss', '.sass', '.less', '.xml', '.yaml', '.yml', '.toml',
  '.sql', '.sh', '.bash', '.zsh', '.env', '.log', '.ini', '.conf', '.config', '.dockerfile',
  '.vue', '.svelte'
]

/**
 * Check if a file is a video
 */
export function isVideoFile(file: File): boolean {
  if (file.type && file.type.startsWith('video/')) return true
  const name = (file.name || '').toLowerCase()
  return VIDEO_EXTENSIONS.some((ext) => name.endsWith(ext))
}

/**
 * Check if a file is an audio
 */
export function isAudioFile(file: File): boolean {
  if (file.type && file.type.startsWith('audio/')) return true
  const name = (file.name || '').toLowerCase()
  return AUDIO_EXTENSIONS.some((ext) => name.endsWith(ext))
}

/**
 * Check if a file is an image
 */
export function isImageFile(file: File): boolean {
  if (file.type && file.type.startsWith('image/')) return true
  const name = (file.name || '').toLowerCase()
  return IMAGE_EXTENSIONS.some((ext) => name.endsWith(ext))
}

export interface ExtractedPdfText {
  text: string
  pages: number
  charCount: number
  isScanned: boolean
}

/**
 * Extract clean text from a PDF file using PDF.js
 */
export async function extractPdfText(file: File): Promise<ExtractedPdfText> {
  try {
    const arrayBuffer = await file.arrayBuffer()
    const loadingTask = pdfjsLib.getDocument({
      data: new Uint8Array(arrayBuffer),
      useWorkerFetch: false,
      isEvalSupported: false,
      useSystemFonts: true
    })

    const pdf = await loadingTask.promise
    const totalPages = pdf.numPages
    let extractedText = ''
    let totalChars = 0

    for (let pageNum = 1; pageNum <= totalPages; pageNum++) {
      const page = await pdf.getPage(pageNum)
      const textContent = await page.getTextContent()
      
      const pageStrings = textContent.items
        .map((item: any) => ('str' in item ? item.str : ''))
        .filter(Boolean)
      
      const pageText = pageStrings.join(' ').replace(/\s+/g, ' ').trim()
      
      if (pageText) {
        extractedText += `--- [Página ${pageNum}/${totalPages}] ---\n${pageText}\n\n`
        totalChars += pageText.length
      }
    }

    if (!extractedText.trim()) {
      return {
        text: `[O arquivo PDF "${file.name}" não contém camada de texto selecionável ou é um documento escaneado/imagem.]`,
        pages: totalPages,
        charCount: 0,
        isScanned: true
      }
    }

    return {
      text: extractedText.trim(),
      pages: totalPages,
      charCount: totalChars,
      isScanned: false
    }
  } catch (err: any) {
    console.error('Erro ao extrair texto do PDF:', err)
    throw new Error(`Falha ao ler PDF: ${err.message || 'Arquivo corrompido ou protegido'}`)
  }
}

export interface TranscriptionProgress {
  status: string
  progress: number
}

export interface TranscriptionOptions {
  model?: string
  language?: string
  [key: string]: any
}

export interface TranscriptionResult {
  text: string
  duration?: number | null
  language?: string | null
}

async function transcribeWithTransformersJs(
  file: File,
  onProgress?: (progress: TranscriptionProgress) => void,
  options: TranscriptionOptions = {}
): Promise<TranscriptionResult> {
  const { pipeline, env } = await import('@xenova/transformers')
  env.allowRemoteModels = true

  const modelId = options.model?.includes('tiny') ? 'Xenova/whisper-tiny' : 'Xenova/whisper-small'

  if (!whisperPipeline) {
    if (onProgress) onProgress({ status: 'downloading', progress: 10 })
    whisperPipeline = await pipeline('automatic-speech-recognition', modelId, {
      progress_callback: (p: any) => {
        if (onProgress && p.status === 'progress') {
          onProgress({ status: 'downloading', progress: Math.round(p.progress || 0) })
        }
      }
    })
  }

  if (onProgress) onProgress({ status: 'decoding_audio', progress: 40 })

  const arrayBuffer = await file.arrayBuffer()
  const audioContext = new (window.AudioContext || (window as any).webkitAudioContext)({ sampleRate: 16000 })
  const audioBuffer = await audioContext.decodeAudioData(arrayBuffer)
  const audioData = audioBuffer.getChannelData(0)

  if (onProgress) onProgress({ status: 'transcribing', progress: 70 })

  const lang = options.language === 'auto' ? null : (options.language || 'portuguese')
  const out: any = await whisperPipeline(audioData, {
    language: lang === 'pt' ? 'portuguese' : lang,
    task: 'transcribe'
  })

  return {
    text: out.text?.trim() || '[Nenhuma fala detectada no arquivo de áudio]',
    duration: audioBuffer.duration || null,
    language: options.language || 'pt'
  }
}

/**
 * Transcribe an audio file using native Whisper (MLX on macOS, faster-whisper on Windows)
 * with graceful client-side WebAssembly fallback (Transformers.js) for universal Windows/Linux support.
 * Supports .opus, .mp3, .wav, .m4a, .ogg, .aac, .weba.
 */
export async function transcribeAudioFile(
  file: File,
  onProgress?: (progress: TranscriptionProgress) => void,
  options: TranscriptionOptions = {}
): Promise<TranscriptionResult> {
  try {
    const { invoke } = await import('@tauri-apps/api/core')

    if (onProgress) onProgress({ status: 'decoding_audio', progress: 20 })

    // Convert File to base64 for Rust backend
    const arrayBuffer = await file.arrayBuffer()
    const uint8 = new Uint8Array(arrayBuffer)
    let binary = ''
    const chunkSize = 8192
    for (let i = 0; i < uint8.length; i += chunkSize) {
      binary += String.fromCharCode.apply(null, Array.from(uint8.subarray(i, i + chunkSize)))
    }
    const audioBase64 = btoa(binary)

    if (onProgress) onProgress({ status: 'transcribing', progress: 50 })

    // 1. Try native backend first (Metal GPU on macOS Apple Silicon, faster-whisper on Windows)
    try {
      const res: any = await invoke('transcribe_audio', {
        audioBase64,
        fileName: file.name,
        language: options.language || 'pt',
        model: options.model || 'mlx-community/whisper-small-mlx'
      })

      return {
        text: res.text || '[Nenhuma fala detectada no arquivo de áudio]',
        duration: res.duration_secs || null,
        language: res.language || null
      }
    } catch (backendErr) {
      // 2. Cross-platform fallback: run locally via WebAssembly Transformers.js
      console.warn('Transcrição nativa não disponível neste sistema. Usando motor cliente Transformers.js:', backendErr)
      return await transcribeWithTransformersJs(file, onProgress, options)
    }
  } catch (err: any) {
    console.error('Erro na transcrição de áudio:', err)
    throw new Error(`Falha ao transcrever áudio: ${err.message || err}`)
  }
}

/**
 * Render a page from a PDF as a base64 image data URL (useful for VLMs when PDF is scanned or image-based)
 */
export async function renderPdfPageToImage(file: File, pageNum = 1): Promise<string | null> {
  try {
    const arrayBuffer = await file.arrayBuffer()
    const loadingTask = pdfjsLib.getDocument({
      data: new Uint8Array(arrayBuffer),
      useWorkerFetch: false,
      isEvalSupported: false,
      useSystemFonts: true
    })
    const pdf = await loadingTask.promise
    if (pageNum < 1 || pageNum > pdf.numPages) return null
    const page = await pdf.getPage(pageNum)
    const viewport = page.getViewport({ scale: 1.5 })
    const canvas = document.createElement('canvas')
    canvas.width = viewport.width
    canvas.height = viewport.height
    const ctx = canvas.getContext('2d')
    if (!ctx) return null
    await page.render({ canvasContext: ctx, viewport }).promise
    return canvas.toDataURL('image/jpeg', 0.85)
  } catch (e) {
    console.warn('Falha ao renderizar página do PDF em imagem:', e)
    return null
  }
}

/**
 * Read clean text from code / text / XML / JSON / CSV file
 */
export async function readTextFile(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.onload = (e) => {
      const content = (e.target?.result as string) || ''
      // Check if file contains excessive null bytes (binary file)
      const sample = content.slice(0, 1000)
      let nullCount = 0
      for (let i = 0; i < sample.length; i++) {
        if (sample.charCodeAt(i) === 0) nullCount++
      }
      if (nullCount > 5) {
        return reject(new Error(`O arquivo "${file.name}" parece ser um arquivo binário e não pôde ser lido como texto.`))
      }
      resolve(content)
    }
    reader.onerror = () => reject(new Error(`Erro ao ler arquivo de texto: ${file.name}`))
    reader.readAsText(file, 'UTF-8')
  })
}
