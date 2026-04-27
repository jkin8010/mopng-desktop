import { useRef, useCallback, type ChangeEvent } from "react";
import { Input } from "@/components/ui/input";
import { cn } from "@/lib/utils";

interface ScrubInputProps {
  value: number;
  onChange: (value: number) => void;
  min?: number;
  max?: number;
  className?: string;
  id?: string;
}

/** Number input that supports left-right drag to adjust values. */
export function ScrubInput({
  value,
  onChange,
  min = 1,
  max = 10000,
  className,
  id,
}: ScrubInputProps) {
  const dragState = useRef<{ startX: number; startValue: number } | null>(null);

  const clamp = (v: number) => Math.min(max, Math.max(min, Math.round(v)));

  const onPointerDown = useCallback(
    (e: React.PointerEvent<HTMLInputElement>) => {
      e.preventDefault();
      (e.target as HTMLInputElement).focus();
      dragState.current = { startX: e.clientX, startValue: value };
      (e.target as HTMLInputElement).setPointerCapture(e.pointerId);
    },
    [value],
  );

  const onPointerMove = useCallback((e: React.PointerEvent<HTMLInputElement>) => {
    if (!dragState.current) return;
    const dx = e.clientX - dragState.current.startX;
    onChange(clamp(dragState.current.startValue + dx));
  }, [onChange, clamp]);

  const onPointerUp = useCallback((e: React.PointerEvent<HTMLInputElement>) => {
    if (!dragState.current) return;
    dragState.current = null;
    (e.target as HTMLInputElement).releasePointerCapture(e.pointerId);
  }, []);

  const handleChange = useCallback(
    (e: ChangeEvent<HTMLInputElement>) => {
      const raw = e.target.value;
      if (raw === "") return;
      const v = parseInt(raw, 10);
      if (!isNaN(v)) onChange(clamp(v));
    },
    [onChange, clamp],
  );

  return (
    <Input
      id={id}
      type="number"
      min={min}
      max={max}
      value={value}
      onChange={handleChange}
      onPointerDown={onPointerDown}
      onPointerMove={onPointerMove}
      onPointerUp={onPointerUp}
      onPointerCancel={onPointerUp}
      className={cn("cursor-ew-resize", className)}
      style={{ cursor: "ew-resize" }}
    />
  );
}
