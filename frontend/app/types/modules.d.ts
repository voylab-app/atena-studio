declare module 'pdfjs-dist/build/pdf.min.mjs' {
  const content: any
  export default content
  export const GlobalWorkerOptions: any
  export function getDocument(params: any): any
}

declare module 'pdfjs-dist/build/pdf.worker.min.mjs?url' {
  const url: string
  export default url
}

declare module '@xenova/transformers' {
  export const pipeline: any
  export const env: any
}

declare module 'highlight.js' {
  const hljs: any
  export default hljs
}

declare module 'markdown-it' {
  const markdownit: any
  export default markdownit
}
