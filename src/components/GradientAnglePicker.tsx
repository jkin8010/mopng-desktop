import { useRef, useCallback, useEffect, useState } from "react";

const VIEWBOX = 100;
const CENTER = VIEWBOX / 2;
const RADIUS = 38;
const HANDLE_R = 7;
const SNAP_ANGLE = 4;

/** 0° = top, clockwise. Returns normalized (x1,y1,x2,y2) through center. */
export function angleToCoords(deg: number): { x1: number; y1: number; x2: number; y2: number } {
  const rad = (deg * Math.PI) / 180;
  const sin = Math.sin(rad);
  const cos = Math.cos(rad);
  return {
    x1: 0.5 - 0.5 * sin,
    y1: 0.5 + 0.5 * cos,
    x2: 0.5 + 0.5 * sin,
    y2: 0.5 - 0.5 * cos,
  };
}

/** Extract angle (0°=top, clockwise) from normalized coords. */
export function coordsToAngle(x1: number, y1: number, x2: number, y2: number): number {
  const dx = x2 - x1;
  const dy = y2 - y1;
  let deg = (Math.atan2(dx, -dy) * 180) / Math.PI;
  if (deg < 0) deg += 360;
  return deg;
}

function snapAngle(angle: number): number {
  const step = 45;
  const remainder = ((angle % step) + step) % step;
  if (remainder <= SNAP_ANGLE || remainder >= step - SNAP_ANGLE) {
    return Math.round(angle / step) * step;
  }
  return angle;
}

interface GradientAnglePickerProps {
  angleDeg: number;
  onAngleChange: (angle: number) => void;
  disabled?: boolean;
}

export function GradientAnglePicker({ angleDeg, onAngleChange, disabled }: GradientAnglePickerProps) {
  const svgRef = useRef<SVGSVGElement>(null);
  const [dragging, setDragging] = useState(false);

  const pointerToAngle = useCallback((clientX: number, clientY: number) => {
    const svg = svgRef.current;
    if (!svg) return angleDeg;
    const rect = svg.getBoundingClientRect();
    const scaleX = VIEWBOX / rect.width;
    const scaleY = VIEWBOX / rect.height;
    const cx = (clientX - rect.left) * scaleX;
    const cy = (clientY - rect.top) * scaleY;
    const dx = cx - CENTER;
    const dy = CENTER - cy;
    let deg = (Math.atan2(dx, dy) * 180) / Math.PI;
    if (deg < 0) deg += 360;
    return deg;
  }, [angleDeg]);

  const onPointerDown = useCallback((e: React.PointerEvent) => {
    if (disabled) return;
    const svg = svgRef.current;
    if (!svg) return;
    svg.setPointerCapture(e.pointerId);
    setDragging(true);
    const raw = pointerToAngle(e.clientX, e.clientY);
    onAngleChange(snapAngle(raw));
  }, [disabled, pointerToAngle, onAngleChange]);

  const onPointerMove = useCallback((e: React.PointerEvent) => {
    if (!dragging || disabled) return;
    const raw = pointerToAngle(e.clientX, e.clientY);
    onAngleChange(snapAngle(raw));
  }, [dragging, disabled, pointerToAngle, onAngleChange]);

  const onPointerUp = useCallback((e: React.PointerEvent) => {
    setDragging(false);
    try {
      svgRef.current?.releasePointerCapture(e.pointerId);
    } catch { /* ignore */ }
  }, []);

  const onKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (disabled) return;
    const step = e.shiftKey ? 5 : 1;
    let next: number | null = null;
    if (e.key === "ArrowUp" || e.key === "ArrowRight") {
      next = snapAngle(angleDeg + step);
    } else if (e.key === "ArrowDown" || e.key === "ArrowLeft") {
      next = snapAngle(angleDeg - step);
    }
    if (next != null) {
      e.preventDefault();
      let n = next;
      if (n >= 360) n -= 360;
      if (n < 0) n += 360;
      onAngleChange(n);
    }
  }, [angleDeg, disabled, onAngleChange]);

  const rad = (angleDeg * Math.PI) / 180;
  const hx = CENTER + RADIUS * Math.sin(rad);
  const hy = CENTER - RADIUS * Math.cos(rad);

  return (
    <svg
      ref={svgRef}
      viewBox={`0 0 ${VIEWBOX} ${VIEWBOX}`}
      className="w-24 h-24 select-none touch-none mx-auto block"
      role="slider"
      aria-valuemin={0}
      aria-valuemax={360}
      aria-valuenow={Math.round(angleDeg)}
      aria-label="渐变角度"
      tabIndex={0}
      onPointerDown={onPointerDown}
      onPointerMove={onPointerMove}
      onPointerUp={onPointerUp}
      onLostPointerCapture={onPointerUp}
      onKeyDown={onKeyDown}
    >
      <circle
        cx={CENTER}
        cy={CENTER}
        r={RADIUS}
        fill="none"
        stroke="hsl(var(--border))"
        strokeWidth="3"
      />
      <line
        x1={CENTER}
        y1={CENTER}
        x2={hx}
        y2={hy}
        stroke="hsl(var(--primary))"
        strokeWidth="1.5"
        strokeLinecap="round"
      />
      <circle
        cx={hx}
        cy={hy}
        r={HANDLE_R}
        fill="hsl(var(--primary))"
        stroke="hsl(var(--background))"
        strokeWidth="2"
        className={dragging ? "" : "transition-[cx,cy] duration-75"}
      />
      <text
        x={CENTER}
        y={CENTER}
        textAnchor="middle"
        dominantBaseline="central"
        className="fill-foreground text-[11px] font-mono"
        style={{ fontSize: "11px" }}
      >
        {Math.round(angleDeg)}°
      </text>
    </svg>
  );
}
