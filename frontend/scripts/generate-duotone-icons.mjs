/**
 * Regenerates src/components/icons/duotone-icons.ts, the icon artwork the whole
 * app renders. Run `npm run icons:generate` after adding a name to ICON_MAP.
 *
 * Why generated: the SVG path data is not ours to write. It is read from the
 * pinned `@iconify-json/solar` package (the Solar icon set, CC-BY-4.0), whose
 * `-duotone` variants are genuine two-layer glyphs -- one full-strength shape
 * plus one at `opacity=".5"`. The app never ships the 8,859-icon collection:
 * only the names listed below are written into the source file, so the bundle
 * carries exactly the artwork that is used and nothing else.
 *
 * The script fails rather than substituting a stand-in when:
 *   - a name imported from 'lucide-react' anywhere in src/ has no mapping, or
 *   - a mapped Solar icon does not exist in the installed collection, or
 *   - a Solar body does not parse into the two-layer shape expected here.
 * A silent fallback would mean shipping an icon that is not the one a page
 * asked for.
 */
import { createRequire } from 'node:module'
import { readdirSync, readFileSync, statSync, writeFileSync } from 'node:fs'
import { dirname, join, relative } from 'node:path'
import { fileURLToPath } from 'node:url'

const require = createRequire(import.meta.url)
const here = dirname(fileURLToPath(import.meta.url))
const frontend = join(here, '..')

const solar = require('@iconify-json/solar/icons.json')

/** Every name the app currently imports from lucide, mapped to a Solar duotone. */
const ICON_MAP = {
  Activity: 'pulse-line-duotone',
  AlertCircle: 'danger-circle-line-duotone',
  AlertTriangle: 'danger-triangle-line-duotone',
  ArrowLeft: 'arrow-left-line-duotone',
  ArrowRight: 'arrow-right-line-duotone',
  Award: 'medal-star-line-duotone',
  Bot: 'bot-line-duotone',
  Briefcase: 'suitcase-line-duotone',
  Building2: 'buildings-line-duotone',
  Camera: 'camera-line-duotone',
  Check: 'check-line-duotone',
  CheckCheck: 'check-read-line-duotone',
  CheckCircle2: 'check-circle-line-duotone',
  ChevronDown: 'alt-arrow-down-line-duotone',
  ChevronUp: 'alt-arrow-up-line-duotone',
  Clock: 'clock-circle-line-duotone',
  Delete: 'backspace-line-duotone',
  DollarSign: 'dollar-line-duotone',
  FileText: 'document-text-line-duotone',
  Flame: 'fire-line-duotone',
  FlaskConical: 'test-tube-line-duotone',
  GitBranch: 'git-branch-line-duotone',
  GitPullRequest: 'git-pull-request-line-duotone',
  Globe: 'earth-line-duotone',
  Handshake: 'hand-shake-line-duotone',
  History: 'history-line-duotone',
  Key: 'key-line-duotone',
  Lightbulb: 'lightbulb-line-duotone',
  Loader2: 'loader-line-duotone',
  Lock: 'shield-keyhole-line-duotone',
  Mail: 'letter-line-duotone',
  Megaphone: 'speaker-line-duotone',
  MessageCircle: 'chat-round-line-duotone',
  MessageSquare: 'chat-square-line-duotone',
  Mic: 'microphone-line-duotone',
  MicOff: 'call-cancel-line-duotone',
  Pause: 'pause-line-duotone',
  Phone: 'phone-line-duotone',
  PhoneCall: 'phone-calling-line-duotone',
  PhoneOff: 'call-dropped-rounded-line-duotone',
  Plus: 'add-line-duotone',
  Radio: 'radio-line-duotone',
  RefreshCw: 'refresh-line-duotone',
  Rocket: 'rocket-line-duotone',
  Save: 'diskette-line-duotone',
  Send: 'send-square-line-duotone',
  Settings: 'settings-line-duotone',
  ShieldCheck: 'shield-check-line-duotone',
  Sliders: 'settings-minimalistic-line-duotone',
  Square: 'frame-line-duotone',
  Star: 'star-line-duotone',
  Tag: 'tag-line-duotone',
  Target: 'target-line-duotone',
  Trash2: 'trash-bin-trash-line-duotone',
  Users: 'users-group-rounded-line-duotone',
  Webhook: 'link-round-angle-line-duotone',
  X: 'close-line-duotone',
  XCircle: 'close-circle-line-duotone',
  Zap: 'bolt-line-duotone',
  ArrowDown: 'arrow-down-line-duotone',
  ArrowDownLeft: 'arrow-left-down-line-duotone',
  ArrowUp: 'arrow-up-line-duotone',
  ArrowUpRight: 'arrow-right-up-line-duotone',
  BadgeCheck: 'verified-check-line-duotone',
  BarChart2: 'chart-line-duotone',
  BarChart3: 'chart-2-line-duotone',
  BookOpen: 'book-2-line-duotone',
  Bookmark: 'bookmark-line-duotone',
  Brain: 'brain-line-duotone',
  Calendar: 'calendar-line-duotone',
  ChevronLeft: 'alt-arrow-left-line-duotone',
  ChevronRight: 'alt-arrow-right-line-duotone',
  CircleDot: 'circle-dot-line-duotone',
  ClipboardList: 'clipboard-line-duotone',
  Code: 'code-line-duotone',
  Copy: 'copy-line-duotone',
  CreditCard: 'card-line-duotone',
  Download: 'download-line-duotone',
  Edit2: 'pen-2-line-duotone',
  Edit3: 'pen-new-round-line-duotone',
  ExternalLink: 'square-arrow-right-up-line-duotone',
  Folder: 'folder-line-duotone',
  GitFork: 'branching-paths-down-line-duotone',
  Hash: 'hashtag-line-duotone',
  Headset: 'headphones-round-line-duotone',
  HelpCircle: 'question-circle-line-duotone',
  Home: 'home-2-line-duotone',
  Image: 'gallery-line-duotone',
  Inbox: 'inbox-line-duotone',
  Info: 'info-circle-line-duotone',
  KeyRound: 'key-minimalistic-line-duotone',
  Languages: 'translation-line-duotone',
  LayoutDashboard: 'layout-list-line-duotone',
  Layers: 'layers-line-duotone',
  Link: 'link-line-duotone',
  Link2: 'link-minimalistic-line-duotone',
  List: 'list-line-duotone',
  ListFilter: 'filter-line-duotone',
  MapPin: 'map-point-line-duotone',
  Monitor: 'monitor-line-duotone',
  Moon: 'moon-line-duotone',
  Music: 'music-note-line-duotone',
  Package: 'box-minimalistic-line-duotone',
  Palette: 'palette-line-duotone',
  Pencil: 'pen-line-duotone',
  Play: 'play-line-duotone',
  RectangleHorizontal: 'text-field-line-duotone',
  Repeat: 'repeat-line-duotone',
  RotateCcw: 'undo-left-round-line-duotone',
  Scroll: 'document-2-line-duotone',
  Search: 'magnifier-line-duotone',
  Scissors: 'scissors-line-duotone',
  Server: 'server-line-duotone',
  Share2: 'share-line-duotone',
  Shield: 'shield-line-duotone',
  ShieldAlert: 'shield-warning-line-duotone',
  Smartphone: 'smartphone-line-duotone',
  Sparkles: 'magic-stick-line-duotone',
  Sun: 'sun-line-duotone',
  ThumbsUp: 'like-line-duotone',
  Ticket: 'ticket-line-duotone',
  ToggleLeft: 'slider-horizontal-line-duotone',
  Truck: 'delivery-line-duotone',
  TrendingUp: 'graph-new-up-line-duotone',
  Upload: 'upload-line-duotone',
  User: 'user-line-duotone',
}

/**
 * Channels that have no brand mark of their own (SMS is a spec, email and
 * voice are protocols) get the duotone glyph for what the channel actually is.
 */
const CHANNEL_MAP = {
  sms: 'chat-square-dots-line-duotone',
  email: 'letter-line-duotone',
  voice: 'phone-calling-line-duotone',
  'live-chat': 'chat-round-dots-line-duotone',
}

const CAMEL = {
  'stroke-width': 'strokeWidth',
  'stroke-linecap': 'strokeLinecap',
  'stroke-linejoin': 'strokeLinejoin',
  'stroke-dasharray': 'strokeDasharray',
  'fill-rule': 'fillRule',
  'clip-rule': 'clipRule',
}

function attributes(raw, sourceName) {
  const out = {}
  for (const [, key, value] of raw.matchAll(/([\w-]+)="([^"]*)"/g)) {
    if (/^[\w-]+$/.test(key) === false) {
      throw new Error(`${sourceName}: attribute name ${key} is not a plain name`)
    }
    out[CAMEL[key] ?? key] = value
  }
  return out
}

const SHAPES = new Set(['path', 'circle', 'rect', 'ellipse', 'line', 'polygon', 'polyline'])
const TOKEN =
  /<(g|path|circle|rect|ellipse|line|polygon|polyline)((?:"[^"]*"|[^>])*?)(\/?)>|<\/\w+>/g

/**
 * Solar bodies are markup, not data, so they are parsed rather than pasted:
 * `<g>` carries the stroke/fill contract and the child shapes carry the
 * `opacity` that makes the glyph duotone. Groups nest (some collections put the
 * faded layer in its own group), so this walks the tree and keeps every node --
 * flattening a nested group onto its children would composite overlapping
 * shapes differently, which is a different drawing.
 *
 * An unknown tag or an empty result is a hard failure: substituting a nearby
 * glyph would ship an icon that is not the one the page asked for.
 */
function parseBody(name, body) {
  const roots = []
  const stack = [{ children: roots }]
  for (const match of body.matchAll(TOKEN)) {
    if (match[0].startsWith('</')) {
      if (stack.length === 1) throw new Error(`${name}: unbalanced closing tag`)
      stack.pop()
      continue
    }
    const [, tag, raw, selfClosing] = match
    const node = { tag, attrs: attributes(raw, name) }
    stack[stack.length - 1].children.push(node)
    if (selfClosing !== '/' && tag === 'g') {
      node.children = []
      stack.push(node)
    }
  }
  if (stack.length !== 1) throw new Error(`${name}: unclosed group`)
  if (!roots.length) throw new Error(`${name}: its body has no shapes`)
  if (!body.includes('opacity')) throw new Error(`${name}: has no faded layer, so it is not duotone`)

  const wrapper = roots.length === 1 && roots[0].tag === 'g' ? roots[0] : null
  return {
    source: `solar:${name}`,
    group: wrapper?.attrs ?? {},
    layers: wrapper ? wrapper.children.map(checkShape) : roots.map(checkShape),
  }
}

function checkShape(node) {
  if (node.children?.length) {
    return { ...node, children: node.children.map(checkShape) }
  }
  if (!SHAPES.has(node.tag)) throw new Error(`cannot render a <${node.tag}> layer`)
  return node
}

function glyph(name) {
  const entry = solar.icons[name]
  if (!entry) throw new Error(`${name} is not in the installed Solar collection`)
  return parseBody(name, entry.body)
}

function sourceFiles(dir) {
  const found = []
  for (const entry of readdirSync(dir)) {
    if (entry === 'node_modules' || entry.startsWith('.')) continue
    const path = join(dir, entry)
    if (statSync(path).isDirectory()) found.push(...sourceFiles(path))
    else if (/\.(ts|tsx)$/.test(entry)) found.push(path)
  }
  return found
}

/**
 * Every icon name the app imports -- from this module now, from lucide-react if
 * a page still does. Either way a name the module cannot supply stops the
 * build, so a page can never render a glyph nobody chose.
 */
function importedIconNames() {
  const names = new Set()
  const srcDir = join(frontend, 'src')
  const iconsDir = join(srcDir, 'components', 'icons')
  for (const path of sourceFiles(srcDir)) {
    // The module's own files define icons; they are not consumers of them.
    if (path.startsWith(iconsDir)) continue
    const text = readFileSync(path, 'utf8')
    for (const [, clause] of text.matchAll(
      /import\s+(?:type\s+)?\{([^}]*)\}\s*from\s*['"][^'"]*(?:lucide-react|\/icons)['"]/g,
    )) {
      for (const raw of clause.split(',')) {
        const name = raw.trim().split(/\s+as\s+/)[0]?.trim()
        if (name && !name.startsWith('type ') && name !== 'CHANNEL_ICONS') names.add(name)
      }
    }
  }
  return names
}

const modulePath = join(frontend, 'src', 'components', 'icons', 'index.tsx')
const moduleSource = readFileSync(modulePath, 'utf8')
/** What the module actually hands out: duotone names, brand marks, channel glyphs. */
const exportedNames = new Set(
  [...moduleSource.matchAll(/^export const (\w+) =/gm)].map(([, name]) => name),
)

const unexported = [...importedIconNames()].filter((name) => !exportedNames.has(name))
if (unexported.length) {
  throw new Error(
    `Pages import these from the icon module, which does not export them: ${unexported.join(', ')}. ` +
      `Add them to ICON_MAP in ${relative(process.cwd(), import.meta.filename)} and export them ` +
      'from src/components/icons/index.tsx, or drop the import.',
  )
}

const icons = Object.entries(ICON_MAP).map(([name, source]) => [name, glyph(source)])
const channels = Object.entries(CHANNEL_MAP).map(([name, source]) => [name, glyph(source)])

const collection = require('@iconify-json/solar/info.json')

const header = `/**
 * GENERATED by scripts/generate-duotone-icons.mjs -- do not edit by hand.
 *
 * Duotone path data for every icon the app renders. The artwork is the
 * ${collection.name} set by ${collection.author.name} (${collection.author.url}),
 * licensed ${collection.license.spdx} (${collection.license.url}) and taken from the
 * pinned \`@iconify-json/solar\` package. frontend/LICENSE-ICONS.md carries the
 * attribution that licence requires.
 *
 * Only the names below are written into source, so the bundle ships the
 * ${icons.length} glyphs this app draws rather than the collection's ${collection.total.toLocaleString('en-US')} icons.
 * Regenerate with \`npm run icons:generate\`; it refuses to write this file when a
 * name has no mapping, the artwork is missing from the collection, or a body
 * does not parse -- a partial table would silently render the wrong icon.
 *
 * Rendered by components/icons/index.tsx, the only module in the app that draws an <svg>.
 */

export type DuotoneNode = {
  tag: 'path' | 'circle' | 'rect' | 'ellipse' | 'line' | 'polygon' | 'polyline' | 'g'
  /** Presentational attributes copied from the collection. Absent means the glyph does not set it. */
  attrs: Record<string, string | undefined>
  /** Present on a group. Its opacity is the faded second tone of the duotone. */
  children?: DuotoneNode[]
}

export type DuotoneGlyph = {
  /** Which icon in the upstream collection this artwork came from. */
  source: string
  /** Attributes the collection puts on the wrapping group (fill, stroke, stroke-width). */
  group: Record<string, string | undefined>
  /** Drawn in order. The duotone effect is whichever node carries \`opacity\`. */
  layers: DuotoneNode[]
}

/** The icon name the page imports, keyed exactly as the previous icon library spelled it. */
export const DUOTONE_ICONS = {`

const body = (entries) =>
  entries
    .map(([name, value]) => {
      const literal = JSON.stringify(value, null, 2)
        .split('\n')
        .map((line, index) => (index === 0 ? line : `  ${line}`))
        .join('\n')
      // 'live-chat' is a channel name, not an identifier, so it needs quoting.
      const key = /^\p{L}[\p{L}\d]*$/u.test(name) ? name : JSON.stringify(name)
      return `  ${key}: ${literal},`
    })
    .join('\n')

const footer = `
} satisfies Record<string, DuotoneGlyph>

export const CHANNEL_GLYPHS = {`

const out = `${header}
${body(icons)}
${footer}
${body(channels)}
} satisfies Record<string, DuotoneGlyph>
`

const target = join(frontend, 'src', 'components', 'icons', 'duotone-icons.ts')

/**
 * Artwork nobody exports is artwork that cannot be used. Checked here, with the
 * data, so a missing export fails the same command that produced it.
 */
const ungenerated = Object.keys(ICON_MAP).filter((name) => !exportedNames.has(name))
if (ungenerated.length) {
  throw new Error(
    `src/components/icons/index.tsx does not export these generated icons: ${ungenerated.join(', ')}. ` +
      `Add the matching \`export const NAME = duotone('NAME')\` line.`,
  )
}

writeFileSync(target, out)
console.log(
  `wrote ${relative(frontend, target)}: ${icons.length} icons, ${channels.length} channel glyphs`,
)
