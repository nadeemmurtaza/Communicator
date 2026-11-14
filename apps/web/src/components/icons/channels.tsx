/**
 * The channel registry: every channel the app can connect, keyed by the name the
 * API returns, mapped to the icon that identifies it.
 *
 * It lives beside the icons rather than inside index.tsx because index.tsx must
 * export components only -- mixing a plain object into it costs those components
 * their Vite fast-refresh eligibility, so editing an icon would reload the app
 * instead of swapping the module.
 *
 * Brand logos stay single-tone (see brand-marks.ts); channels that are
 * protocols -- SMS, email, voice, live chat -- get the duotone glyph for what
 * they actually are.
 */
import type { ReactElement, SVGProps } from 'react'
import {
  EmailIcon,
  InstagramIcon,
  FacebookIcon,
  LineIcon,
  LiveChatIcon,
  SmsIcon,
  TelegramIcon,
  TikTokIcon,
  VoiceIcon,
  WhatsAppIcon,
} from './index'

type IconComponent = (props: SVGProps<SVGSVGElement>) => ReactElement

export const CHANNEL_ICONS: Record<string, IconComponent> = {
  whatsapp: WhatsAppIcon,
  instagram: InstagramIcon,
  facebook: FacebookIcon,
  telegram: TelegramIcon,
  tiktok: TikTokIcon,
  line: LineIcon,
  sms: SmsIcon,
  email: EmailIcon,
  voice: VoiceIcon,
  'live-chat': LiveChatIcon,
}
