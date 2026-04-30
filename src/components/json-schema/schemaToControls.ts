/**
 * Maps a JSON Schema param_schema to an array of ControlDescriptor objects.
 * Pure function — no side effects, no React imports.
 *
 * Per D-13 mapping rules:
 * - {type: "number", minimum, maximum}  => "slider"
 * - {type: "integer"}                    => "number"
 * - {type: "boolean"}                    => "switch"
 * - {type: "string", enum: [...]}        => "select"
 * - {type: "string"} (no enum)           => "text"
 */

export interface ControlDescriptor {
  name: string;
  label: string;
  controlType: "slider" | "select" | "switch" | "number" | "text";
  value: unknown;
  min?: number;
  max?: number;
  step?: number;
  options?: { label: string; value: string }[];
  description?: string;
}

type JsonSchemaProperty = {
  type?: string;
  title?: string;
  description?: string;
  minimum?: number;
  maximum?: number;
  default?: unknown;
  enum?: string[];
  multipleOf?: number;
};

type ParamSchema = {
  type?: string;
  properties?: Record<string, JsonSchemaProperty>;
  "x-order"?: string[];
};

/**
 * Convert a param_schema + current values into an ordered array of ControlDescriptors.
 */
export function schemaToControls(
  schema: ParamSchema | null | undefined,
  currentValues: Record<string, unknown>
): ControlDescriptor[] {
  if (!schema || !schema.properties) return [];

  const properties = schema.properties;
  // Use explicit ordering if provided, otherwise fall back to Object.entries
  // Per Pitfall 4: JSON Schema objects are unordered; x-order provides explicit ordering
  const order = schema["x-order"] ?? Object.keys(properties);

  return order
    .filter((name) => name in properties)
    .map((name) => {
      const prop = properties[name];
      const type = prop.type;
      const desc: ControlDescriptor = {
        name,
        label: prop.title ?? toLabel(name),
        controlType: "text",
        value: currentValues[name] ?? prop.default ?? "",
        description: prop.description,
      };

      if (type === "number") {
        desc.controlType = "slider";
        desc.min = prop.minimum ?? 0;
        desc.max = prop.maximum ?? 1;
        desc.step = prop.multipleOf ?? 0.01;
        desc.value = (currentValues[name] ?? prop.default ?? desc.min) as number;
      } else if (type === "integer") {
        desc.controlType = "number";
        desc.min = prop.minimum ?? 0;
        desc.max = prop.maximum ?? 100;
        desc.step = 1;
        desc.value = (currentValues[name] ?? prop.default ?? desc.min) as number;
      } else if (type === "boolean") {
        desc.controlType = "switch";
        desc.value = (currentValues[name] ?? prop.default ?? false) as boolean;
      } else if (type === "string" && Array.isArray(prop.enum)) {
        desc.controlType = "select";
        desc.options = prop.enum.map((v) => ({ label: v, value: v }));
        desc.value = (currentValues[name] ?? prop.default ?? prop.enum[0]) as string;
      }
      // else: type === "string" (no enum) → controlType stays "text"

      return desc;
    });
}

/** Convert PascalCase/camelCase key to human-readable label fallback. */
function toLabel(key: string): string {
  // Insert space before uppercase letters, capitalize first letter
  return key
    .replace(/([A-Z])/g, " $1")
    .replace(/^./, (s) => s.toUpperCase())
    .trim();
}
