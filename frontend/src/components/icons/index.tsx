/**
 * The app's only icon module.
 *
 * Every page, card, row and button takes its icons from here -- `import { X,
 * RefreshCw } from '../components/icons'` -- and `Glyph` below is the only place
 * in the project that draws its own `<svg>`. Nothing else writes SVG markup, so a
 * glyph cannot drift between two pages and the artwork has one place to change.
 *
 * The style is duotone: each glyph is drawn in two tones, a full-strength shape
 * and a companion at half opacity, exactly as the collection authors it (see
 * duotone-icons.ts for where the path data comes from and the licence that
 * requires attribution). The tones come from the artwork, not from a filter, so
 * `text-*` recolours an icon and `h-4 w-4` sizes it -- both behave as they did
 * with the previous icon library.
 *
 * Exception: the channel logos in brand-marks.ts stay single-tone. Fading half a
 * trademark would misrepresent the mark; those paths are the official ones.
 *
 * The module is five files, one of which you import from:
 *   index.tsx        -- this file, every icon component and the only hand-drawn
 *                       <svg>; re-exports the animated set from animated.tsx
 *   channels.tsx     -- the CHANNEL_ICONS registry, kept out of here so this
 *                       file exports components only and stays fast-refreshable
 *   duotone-icons.ts -- generated duotone path data, from npm run icons:generate
 *   brand-marks.ts   -- the official channel logo paths
 *   animated.tsx     -- the handful of icons that move; lucide-animated draws
 *                       their svg, so sizing and stroke go through `size` and
 *                       `stroke`, never `className` width/height utilities
 *
 * To add an icon: put its name in ICON_MAP in scripts/generate-duotone-icons.mjs,
 * run `npm run icons:generate`, and add its one-line component below. The
 * generator checks both directions -- a name imported anywhere with no artwork
 * fails, and artwork with no export here fails -- so the table and this file
 * cannot drift apart.
 */
import { createElement, type ReactElement, type SVGProps } from 'react'
import { BRAND_GLYPHS } from './brand-marks'
import { CHANNEL_GLYPHS, DUOTONE_ICONS, type DuotoneGlyph, type DuotoneNode } from './duotone-icons'
// Icons not in the duotone set — re-exported directly from lucide-react
import { ArrowRightLeft, Ban, BarChart, Bell, CheckCircle, Cpu, Database, Eye, EyeOff, Gauge, ListTodo, Minus, TrendingDown, Video, Warehouse } from 'lucide-react'
export { ArrowRightLeft, Ban, BarChart, Bell, CheckCircle, Cpu, Database, Eye, EyeOff, Gauge, ListTodo, Minus, TrendingDown, Video, Warehouse }

/**
 * Not exported on purpose: a module whose only extra export is a type still
 * loses fast refresh for its components, and call sites pass `className`, which
 * every element accepts anyway.
 */
type IconProps = SVGProps<SVGSVGElement>

function renderNodes(nodes: readonly DuotoneNode[], path: string): ReactElement[] {
  return nodes.map((node, index) => {
    const key = `${path}-${index}`
    // These attributes are build-time data copied from a pinned package, not
    // runtime input, so each node becomes an element rather than injected markup.
    const props = { key, ...node.attrs }
    return node.children
      ? createElement(node.tag, props, renderNodes(node.children, key))
      : createElement(node.tag, props)
  })
}

/**
 * The single <svg> of the app. `width`/`height` are defaults, not constraints:
 * a Tailwind `h-4 w-4` on `className` overrides them, which is how every call
 * site already sizes its icons.
 */
function Glyph({ glyph, className, ...rest }: IconProps & { glyph: DuotoneGlyph }): ReactElement {
  return (
    <svg
      viewBox="0 0 24 24"
      width={24}
      height={24}
      fill="none"
      className={className}
      aria-hidden="true"
      focusable="false"
      {...rest}
    >
      {createElement('g', { key: 'tone', ...glyph.group }, renderNodes(glyph.layers, 'n'))}
    </svg>
  )
}

export const Activity = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Activity} {...props} />
export const AlertCircle = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.AlertCircle} {...props} />
export const AlertTriangle = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.AlertTriangle} {...props} />
export const ArrowDown = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.ArrowDown} {...props} />
export const ArrowDownLeft = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.ArrowDownLeft} {...props} />
export const ArrowLeft = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.ArrowLeft} {...props} />
export const ArrowRight = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.ArrowRight} {...props} />
export const ArrowUp = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.ArrowUp} {...props} />
export const ArrowUpRight = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.ArrowUpRight} {...props} />
export const Award = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Award} {...props} />
export const BadgeCheck = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.BadgeCheck} {...props} />
export const BarChart2 = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.BarChart2} {...props} />
export const BarChart3 = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.BarChart3} {...props} />
export const BookOpen = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.BookOpen} {...props} />
export const Bookmark = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Bookmark} {...props} />
export const Bot = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Bot} {...props} />
export const Brain = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Brain} {...props} />
export const Briefcase = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Briefcase} {...props} />
export const Building2 = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Building2} {...props} />
export const Calendar = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Calendar} {...props} />
export const Camera = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Camera} {...props} />
export const Check = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Check} {...props} />
export const CheckCheck = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.CheckCheck} {...props} />
export const CheckCircle2 = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.CheckCircle2} {...props} />
export const ChevronDown = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.ChevronDown} {...props} />
export const ChevronLeft = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.ChevronLeft} {...props} />
export const ChevronRight = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.ChevronRight} {...props} />
export const ChevronUp = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.ChevronUp} {...props} />
export const CircleDot = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.CircleDot} {...props} />
export const ClipboardList = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.ClipboardList} {...props} />
export const Clock = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Clock} {...props} />
export const Code = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Code} {...props} />
export const Copy = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Copy} {...props} />
export const CreditCard = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.CreditCard} {...props} />
export const Delete = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Delete} {...props} />
export const DollarSign = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.DollarSign} {...props} />
export const Download = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Download} {...props} />
export const Edit2 = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Edit2} {...props} />
export const Edit3 = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Edit3} {...props} />
export const ExternalLink = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.ExternalLink} {...props} />
export const FileText = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.FileText} {...props} />
export const Flame = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Flame} {...props} />
export const FlaskConical = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.FlaskConical} {...props} />
export const Folder = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Folder} {...props} />
export const GitBranch = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.GitBranch} {...props} />
export const GitFork = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.GitFork} {...props} />
export const GitPullRequest = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.GitPullRequest} {...props} />
export const Globe = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Globe} {...props} />
export const Handshake = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Handshake} {...props} />
export const Hash = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Hash} {...props} />
export const Headset = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Headset} {...props} />
export const HelpCircle = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.HelpCircle} {...props} />
export const History = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.History} {...props} />
export const Home = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Home} {...props} />
export const Image = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Image} {...props} />
export const Inbox = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Inbox} {...props} />
export const Info = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Info} {...props} />
export const Key = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Key} {...props} />
export const KeyRound = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.KeyRound} {...props} />
export const Languages = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Languages} {...props} />
export const Layers = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Layers} {...props} />
export const LayoutDashboard = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.LayoutDashboard} {...props} />
export const Lightbulb = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Lightbulb} {...props} />
export const Link = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Link} {...props} />
export const Link2 = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Link2} {...props} />
export const List = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.List} {...props} />
export const ListFilter = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.ListFilter} {...props} />
export const Loader2 = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Loader2} {...props} />
export const Lock = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Lock} {...props} />
export const Mail = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Mail} {...props} />
export const MapPin = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.MapPin} {...props} />
export const Megaphone = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Megaphone} {...props} />
export const MessageCircle = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.MessageCircle} {...props} />
export const MessageSquare = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.MessageSquare} {...props} />
export const Mic = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Mic} {...props} />
export const MicOff = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.MicOff} {...props} />
export const Monitor = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Monitor} {...props} />
export const Moon = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Moon} {...props} />
export const Music = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Music} {...props} />
export const Package = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Package} {...props} />
export const Palette = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Palette} {...props} />
export const Pause = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Pause} {...props} />
export const Pencil = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Pencil} {...props} />
export const Phone = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Phone} {...props} />
export const PhoneCall = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.PhoneCall} {...props} />
export const PhoneOff = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.PhoneOff} {...props} />
export const Play = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Play} {...props} />
export const Plus = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Plus} {...props} />
export const Radio = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Radio} {...props} />
export const RectangleHorizontal = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.RectangleHorizontal} {...props} />
export const RefreshCw = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.RefreshCw} {...props} />
export const Repeat = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Repeat} {...props} />
export const Rocket = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Rocket} {...props} />
export const RotateCcw = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.RotateCcw} {...props} />
export const Save = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Save} {...props} />
export const Scissors = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Scissors} {...props} />
export const Scroll = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Scroll} {...props} />
export const Search = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Search} {...props} />
export const Send = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Send} {...props} />
export const Server = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Server} {...props} />
export const Settings = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Settings} {...props} />
export const Share2 = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Share2} {...props} />
export const Shield = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Shield} {...props} />
export const ShieldAlert = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.ShieldAlert} {...props} />
export const ShieldCheck = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.ShieldCheck} {...props} />
export const Sliders = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Sliders} {...props} />
export const Smartphone = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Smartphone} {...props} />
export const Sparkles = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Sparkles} {...props} />
export const Square = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Square} {...props} />
export const Star = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Star} {...props} />
export const Sun = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Sun} {...props} />
export const Tag = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Tag} {...props} />
export const Target = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Target} {...props} />
export const ThumbsUp = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.ThumbsUp} {...props} />
export const Ticket = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Ticket} {...props} />
export const ToggleLeft = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.ToggleLeft} {...props} />
export const Trash2 = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Trash2} {...props} />
export const TrendingUp = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.TrendingUp} {...props} />
export const Truck = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Truck} {...props} />
export const Upload = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Upload} {...props} />
export const User = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.User} {...props} />
export const Users = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Users} {...props} />
export const Webhook = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Webhook} {...props} />
export const X = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.X} {...props} />
export const XCircle = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.XCircle} {...props} />
export const Zap = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Zap} {...props} />
export const ShoppingCart = (props: IconProps) => <Glyph glyph={DUOTONE_ICONS.Package} {...props} />

export const WhatsAppIcon = (props: IconProps) => <Glyph glyph={BRAND_GLYPHS.whatsapp} {...props} />
export const InstagramIcon = (props: IconProps) => <Glyph glyph={BRAND_GLYPHS.instagram} {...props} />
export const FacebookIcon = (props: IconProps) => <Glyph glyph={BRAND_GLYPHS.facebook} {...props} />
export const TelegramIcon = (props: IconProps) => <Glyph glyph={BRAND_GLYPHS.telegram} {...props} />
export const TikTokIcon = (props: IconProps) => <Glyph glyph={BRAND_GLYPHS.tiktok} {...props} />
export const LineIcon = (props: IconProps) => <Glyph glyph={BRAND_GLYPHS.line} {...props} />
export const DiscordIcon = (props: IconProps) => <Glyph glyph={BRAND_GLYPHS.discord} {...props} />
export const SlackIcon = (props: IconProps) => <Glyph glyph={BRAND_GLYPHS.slack} {...props} />

/** Channels with no logo of their own: SMS, email and voice are protocols. */
export const SmsIcon = (props: IconProps) => <Glyph glyph={CHANNEL_GLYPHS.sms} {...props} />
export const EmailIcon = (props: IconProps) => <Glyph glyph={CHANNEL_GLYPHS.email} {...props} />
export const VoiceIcon = (props: IconProps) => <Glyph glyph={CHANNEL_GLYPHS.voice} {...props} />
export const LiveChatIcon = (props: IconProps) => (
  <Glyph glyph={CHANNEL_GLYPHS['live-chat']} {...props} />
)

// Named, not `export *`, so this file keeps exporting components only.
export {
  FlameAnimated,
  MessageSquareAnimated,
  ShieldCheckAnimated,
  SparklesAnimated,
} from './animated'
// Erased at build time, so it cannot cost this file its fast refresh.
export type { AnimatedIconHandle, AnimatedIconProps } from './animated'
