/**
 * Animated icons, reached through the same import as every other glyph.
 *
 * These are the only icons in the app that move, and they are deliberately
 * few: motion on a page that already has a spinner, a live-connection dot and
 * a refresh button is noise, so the set is opt-in per call site rather than a
 * drop-in replacement for the duotone exports in `index.tsx`.
 *
 * They come from `lucide-animated` (the pqoqubbw/icons collection, MIT; see
 * LICENSE-ICONS.md). That library renders its own `<svg>`, which is the one
 * exception to `index.tsx`'s "Glyph is the only place that draws an svg" rule:
 * the artwork is driven by `motion` keyframes on individual paths, so it cannot
 * be expressed as static path data in `duotone-icons.ts`.
 *
 * The pixel-quality rules, all enforced here so no call site has to know them:
 *
 * 1. `size` is rounded to an integer CSS pixel. A 20.5px box puts a hairline
 *    across two device rows and the browser antialiases it into grey fuzz.
 * 2. Stroke weight is authored in *device* pixels, then converted to the
 *    library's 24-unit viewBox (`stroke * 24 / size`). At the default 20px box
 *    that is 2.4 units -- exactly 2 real pixels of ink. The raw `stroke-width`
 *    attribute is always 2 in this library, and 2 units at 20px is 1.67px,
 *    which is what makes small animated icons look soft.
 * 3. The stroke has to be set from CSS: the library spreads props onto the
 *    wrapper `div`, never the inner `<svg>`, so `strokeWidth` is unreachable as
 *    a prop. A CSS `stroke-width` does beat the presentation attribute, which
 *    is what `.icon-animated > svg` in `index.css` is for.
 * 4. `ref` changes the library's behaviour, so this module owns it: passing any
 *    ref makes the icon "controlled", which disables its own hover handling. The
 *    wrapper only attaches the internal ref when the caller asked for one, and
 *    keeps native hover otherwise.
 * 5. `prefers-reduced-motion` is not honoured by the library (it animates from
 *    JS, outside the reach of the CSS reset in `index.css`), so both the hover
 *    prop and the imperative handle gate on it here.
 */
import {
  forwardRef,
  useImperativeHandle,
  useRef,
  type CSSProperties,
  type ForwardedRef,
  type HTMLAttributes,
  type RefObject,
} from 'react'
import {
  FlameIcon,
  MessageSquareIcon,
  ShieldCheckIcon,
  SparklesIcon,
  type FlameIconHandle,
  type MessageSquareIconHandle,
  type ShieldCheckIconHandle,
  type SparklesIconHandle,
} from 'lucide-animated'

/** What `ref` gives you: play the animation from a parent, e.g. a card hover. */
export interface AnimatedIconHandle {
  startAnimation: () => void
  stopAnimation: () => void
}

/**
 * Div attributes, not `SVGProps`: the animated artwork is wrapped in a `div` by
 * the library, so `className` lands there and `width`/`height` are meaningless.
 * Size through `size`, colour through `text-*` as usual.
 */
export interface AnimatedIconProps extends HTMLAttributes<HTMLDivElement> {
  /** Box in CSS pixels, rounded to a whole pixel before it reaches the svg. */
  size?: number
  /** Stroke weight in *device* pixels, converted to viewBox units for the svg. */
  stroke?: number
  /** Play on the icon's own pointer enter. Default true; ignored when `ref` is used. */
  animateOnHover?: boolean
}

const VIEW_BOX = 24

function strokeUnits(px: number, devicePx: number): number {
  return (devicePx * VIEW_BOX) / px
}

function prefersReducedMotion(): boolean {
  return (
    typeof window !== 'undefined' &&
    window.matchMedia('(prefers-reduced-motion: reduce)').matches
  )
}

function frameProps(props: AnimatedIconProps, controlled: boolean) {
  const { size = 20, stroke = 2, className, style, animateOnHover = true, ...rest } = props
  const px = Number.isFinite(size) ? Math.max(8, Math.round(size)) : 20
  return {
    ...rest,
    size: px,
    animateOnHover: controlled ? false : animateOnHover && !prefersReducedMotion(),
    className: ['icon-animated', className].filter(Boolean).join(' '),
    style: { '--icon-stroke': strokeUnits(px, stroke), ...style } as CSSProperties,
  }
}

/**
 * The caller's ref is mirrored onto the library's handle so reduced motion can
 * veto `startAnimation`; `inner` stays null when nobody asked to control the
 * icon, which is what leaves native hover working.
 */
function useHandle<T extends AnimatedIconHandle>(
  ref: ForwardedRef<AnimatedIconHandle>,
): RefObject<T | null> {
  const inner = useRef<T>(null)
  useImperativeHandle(ref ?? null, () => ({
    startAnimation: () => {
      if (!prefersReducedMotion()) inner.current?.startAnimation()
    },
    stopAnimation: () => inner.current?.stopAnimation(),
  }))
  return inner
}

export const SparklesAnimated = forwardRef<AnimatedIconHandle, AnimatedIconProps>(
  function SparklesAnimated(props, ref) {
    const handle = useHandle<SparklesIconHandle>(ref)
    return <SparklesIcon ref={ref ? handle : undefined} {...frameProps(props, !!ref)} />
  },
)

export const FlameAnimated = forwardRef<AnimatedIconHandle, AnimatedIconProps>(
  function FlameAnimated(props, ref) {
    const handle = useHandle<FlameIconHandle>(ref)
    return <FlameIcon ref={ref ? handle : undefined} {...frameProps(props, !!ref)} />
  },
)

export const MessageSquareAnimated = forwardRef<AnimatedIconHandle, AnimatedIconProps>(
  function MessageSquareAnimated(props, ref) {
    const handle = useHandle<MessageSquareIconHandle>(ref)
    return <MessageSquareIcon ref={ref ? handle : undefined} {...frameProps(props, !!ref)} />
  },
)

export const ShieldCheckAnimated = forwardRef<AnimatedIconHandle, AnimatedIconProps>(
  function ShieldCheckAnimated(props, ref) {
    const handle = useHandle<ShieldCheckIconHandle>(ref)
    return <ShieldCheckIcon ref={ref ? handle : undefined} {...frameProps(props, !!ref)} />
  },
)
