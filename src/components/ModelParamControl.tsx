import { Slider } from "@/components/ui/slider";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import type { ControlDescriptor } from "@/components/json-schema/schemaToControls";

interface ModelParamControlProps {
  control: ControlDescriptor;
  onChange: (name: string, value: unknown) => void;
}

export function ModelParamControl({ control, onChange }: ModelParamControlProps) {
  const { name, label, controlType, value, min, max, step, options, description } = control;

  return (
    <div className="space-y-1.5">
      <div className="flex items-center justify-between">
        <Label htmlFor={`param-${name}`} className="text-xs font-medium text-muted-foreground">
          {label}
        </Label>
        {(controlType === "slider" || controlType === "number") && (
          <span className="text-xs font-mono text-muted-foreground">
            {typeof value === "number" ? value.toFixed(controlType === "number" ? 0 : 2) : String(value)}
          </span>
        )}
      </div>

      {controlType === "slider" && (
        <Slider
          id={`param-${name}`}
          value={[typeof value === "number" ? value : (min ?? 0)]}
          onValueChange={([v]) => onChange(name, v)}
          min={min ?? 0}
          max={max ?? 1}
          step={step ?? 0.01}
        />
      )}

      {controlType === "select" && (
        <Select value={String(value ?? "")} onValueChange={(v) => onChange(name, v)}>
          <SelectTrigger id={`param-${name}`} className="h-8 text-xs">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {options?.map((opt) => (
              <SelectItem key={opt.value} value={opt.value}>
                {opt.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      )}

      {controlType === "switch" && (
        <div className="flex items-center gap-2">
          <Switch
            id={`param-${name}`}
            checked={!!value}
            onCheckedChange={(v) => onChange(name, v)}
          />
          <Label
            htmlFor={`param-${name}`}
            className="text-xs text-muted-foreground cursor-pointer"
          >
            {value ? "开启" : "关闭"}
          </Label>
        </div>
      )}

      {controlType === "number" && (
        <Input
          id={`param-${name}`}
          type="number"
          value={typeof value === "number" ? value : (min ?? 0)}
          onChange={(e) => {
            const v = e.target.value === "" ? (min ?? 0) : Number(e.target.value);
            if (!isNaN(v)) onChange(name, v);
          }}
          min={min}
          max={max}
          step={step ?? 1}
          className="h-8 text-xs"
        />
      )}

      {controlType === "text" && (
        <Input
          id={`param-${name}`}
          type="text"
          value={String(value ?? "")}
          onChange={(e) => onChange(name, e.target.value)}
          className="h-8 text-xs"
        />
      )}

      {description && (
        <p className="text-[11px] text-muted-foreground leading-tight">{description}</p>
      )}
    </div>
  );
}
