/**
 * Utilities for normalizing and contracting file system paths.
 * In accordance with privacy and multi-user guidelines, paths inside the
 * user's home directory are displayed and stored using the '~/' prefix.
 */

export function contractUserPath(path: string | null | undefined): string {
  if (!path || typeof path !== 'string') return ''
  const trimmed = path.trim()
  if (!trimmed || trimmed.startsWith('~')) return trimmed

  // Match /Users/<user>, /home/<user>, and macOS /System/Volumes/Data/Users/<user>
  const unixMatch = trimmed.replace(/^\/(?:System\/Volumes\/Data\/)?(?:Users|home)\/[^/]+/, '~')
  if (unixMatch !== trimmed) {
    return unixMatch
  }

  // Match Windows C:\Users\<user>
  const winMatch = trimmed.replace(/^[A-Za-z]:\\(?:Users)\\[^\\]+/, '~')
  if (winMatch !== trimmed) {
    return winMatch.replace(/\\/g, '/')
  }

  return trimmed
}
