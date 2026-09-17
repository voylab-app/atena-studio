/**
 * Composable for resolving authentic brand/family logos for models.
 * When a model comes from an aggregator/quantizer (e.g. lmstudio-community, mlx-community,
 * bartowski, thebloke, unsloth) or is clearly identifiable as a foundation model family
 * (Gemma, Qwen, Llama, DeepSeek, Mistral, Phi), it prioritizes the official model logo
 * instead of displaying the repackager's logo (such as LM Studio).
 */

export interface ModelLogoTarget {
  id?: string | null
  name?: string | null
  author?: string | null
  architecture?: string | null
  local_path?: string | null
  author_avatar_url?: string | null
  backend?: string | null
  format?: string | null
}

const COMMUNITY_QUANTIZERS = new Set([
  'lmstudio-community',
  'lmstudio',
  'lm-studio',
  'mlx-community',
  'bartowski',
  'thebloke',
  'unsloth',
  'mradermacher',
  'city96',
  'local',
  'models'
])

export const isCommunityQuantizer = (author?: string | null): boolean => {
  if (!author) return true
  const lower = author.trim().toLowerCase()
  if (COMMUNITY_QUANTIZERS.has(lower)) return true
  if (lower.includes('lmstudio') || lower.includes('lm-studio')) return true
  return false
}

/**
 * Returns the authentic family logo URL (SVG/PNG) for a model if identified,
 * otherwise null.
 */
export const getModelFamilyLogo = (model?: ModelLogoTarget | null): string | null => {
  if (!model) return null

  const id = (model.id || '').toLowerCase()
  const name = (model.name || '').toLowerCase()
  const arch = (model.architecture || '').toLowerCase()
  const path = (model.local_path || '').toLowerCase()
  const author = (model.author || '').toLowerCase()

  const text = `${id} ${name} ${path}`

  // 1. Google Gemma
  // Matches "gemma", "gemma2", "gemma3", "gemma4", "codegemma"
  if (
    text.includes('gemma') ||
    arch.includes('gemma') ||
    author === 'google' ||
    author === 'google-deepmind'
  ) {
    if (text.includes('gemma') || arch.includes('gemma')) {
      return '/icons/models/gemma.svg'
    }
    if (text.includes('gemini')) {
      return '/icons/models/gemini.svg'
    }
  }

  // 2. Alibaba Qwen
  // Matches "qwen", "qwen2", "qwen2.5", "qwen3", "qwen3.5"
  // Note: custom projects like ornith-ai or prism-ml have their own names;
  // if author is a community quantizer OR name/id contains qwen, it's Qwen!
  if (
    text.includes('qwen') ||
    author === 'qwen' ||
    author === 'alibaba' ||
    author === 'alibaba-nlp' ||
    (arch.includes('qwen') && isCommunityQuantizer(author))
  ) {
    return '/icons/models/qwen.svg'
  }

  // 3. DeepSeek
  if (
    text.includes('deepseek') ||
    author.includes('deepseek') ||
    arch.includes('deepseek')
  ) {
    return '/icons/models/deepseek.svg'
  }

  // 4. Mistral / Codestral / Pixtral / Ministral / Mixtral
  if (
    text.includes('mistral') ||
    text.includes('codestral') ||
    text.includes('pixtral') ||
    text.includes('ministral') ||
    text.includes('mixtral') ||
    author === 'mistralai' ||
    author === 'mistral'
  ) {
    return '/icons/models/mistral.svg'
  }

  // 5. Meta / Llama
  if (
    text.includes('llama') ||
    author === 'meta-llama' ||
    author === 'meta' ||
    author === 'facebook' ||
    (arch.includes('llama') && isCommunityQuantizer(author))
  ) {
    return '/icons/models/meta.svg'
  }

  // 6. Microsoft Phi
  if (
    text.includes('phi-') ||
    text.includes('phi2') ||
    text.includes('phi3') ||
    text.includes('phi4') ||
    text.includes('phi_') ||
    (arch.includes('phi') && isCommunityQuantizer(author))
  ) {
    return '/icons/models/microsoft.svg'
  }

  // 7. Gemini
  if (text.includes('gemini')) {
    return '/icons/models/gemini.svg'
  }

  return null
}

/**
 * Resolves the best logo for a model:
 * 1. Checks if the model belongs to a known model family (Gemma, Qwen, etc.).
 *    If the model is Gemma or Qwen (or similar) and either:
 *    - The author is a quantizer/community repo (like lmstudio-community, mlx-community)
 *    - Or the author has no custom logo
 *    - Or the name explicitly identifies the model family
 *    -> Uses the model family logo!
 * 2. If author is a creator/organization with a custom avatar (e.g. ornith-ai, prism-ml, nvidia),
 *    uses their authentic avatar.
 * 3. Falls back to dynamic catalog avatars or author_avatar_url.
 */
export const resolveModelLogo = (
  model?: ModelLogoTarget | null,
  dynamicAvatarMap?: Map<string, string | null>
): string | null => {
  if (!model) return null

  const familyLogo = getModelFamilyLogo(model)
  const authorLower = (model.author || '').trim().toLowerCase()
  const isQuantizer = isCommunityQuantizer(authorLower)

  // If we found a family logo AND the author is a quantizer/community repo
  // (e.g. lmstudio-community, mlx-community, etc.) OR author is the creator themselves:
  // The family logo takes absolute precedence over the quantizer's logo!
  if (
    familyLogo &&
    (isQuantizer ||
      authorLower === 'qwen' ||
      authorLower === 'google' ||
      authorLower === 'meta' ||
      authorLower === 'meta-llama' ||
      authorLower === 'mistralai' ||
      authorLower === 'deepseek-ai')
  ) {
    return familyLogo
  }

  // If the model name or ID explicitly contains "gemma" or "qwen":
  // Directly satisfies: "quando identificar a llm como Gemma ou Qwen ele colocar a logo do modelo"
  const nameOrId = `${model.id || ''} ${model.name || ''}`.toLowerCase()
  if (familyLogo && (nameOrId.includes('gemma') || nameOrId.includes('qwen'))) {
    return familyLogo
  }

  // If model already has a specific author_avatar_url from a non-quantizer creator (e.g. ornith-ai, prism-ml, nvidia)
  if (model.author_avatar_url && !isQuantizer) {
    return model.author_avatar_url
  }

  // Check dynamic fetched avatars from Hugging Face for non-quantizers
  if (dynamicAvatarMap && dynamicAvatarMap.has(authorLower)) {
    const dynamicUrl = dynamicAvatarMap.get(authorLower)
    if (dynamicUrl && !isQuantizer) return dynamicUrl
  }

  // If it was from a quantizer but we have family logo, use it
  if (familyLogo) {
    return familyLogo
  }

  // Fallbacks
  if (model.author_avatar_url) {
    return model.author_avatar_url
  }

  if (dynamicAvatarMap && dynamicAvatarMap.has(authorLower)) {
    const dynamicUrl = dynamicAvatarMap.get(authorLower)
    if (dynamicUrl) return dynamicUrl
  }

  if (authorLower === 'mlx-community') {
    return 'https://cdn-avatars.huggingface.co/v1/production/uploads/623c830997ddced06d78699b/3qTjC7d3YFCJTwpxd2noq.png'
  }

  return null
}
