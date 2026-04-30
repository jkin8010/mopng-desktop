import { useMemo } from "react";
import { ChevronDown } from "lucide-react";
import { useStore } from "@/store";
import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from "@/components/ui/accordion";
import { ModelParamControl } from "@/components/ModelParamControl";
import { schemaToControls } from "@/components/json-schema/schemaToControls";

export function ModelParamSection() {
  const availableModels = useStore((s) => s.availableModels);
  const activeModelId = useStore((s) => s.activeModelId);
  const modelParams = useStore((s) => s.modelParams);
  const setModelParams = useStore((s) => s.setModelParams);

  const activeModel = availableModels.find((m) => m.id === activeModelId);
  const paramSchema = activeModel?.paramSchema;

  const controls = useMemo(
    () => schemaToControls(paramSchema as Parameters<typeof schemaToControls>[0], modelParams ?? {}),
    [paramSchema, modelParams]
  );

  // Don't render if no parameters
  if (controls.length === 0) return null;

  const handleParamChange = (name: string, value: unknown) => {
    setModelParams({ ...(modelParams ?? {}), [name]: value });
  };

  return (
    <Accordion type="single" defaultValue="model-params" collapsible>
      <AccordionItem value="model-params" className="border-b-0">
        <AccordionTrigger className="group py-2 text-xs font-semibold hover:no-underline [&>svg]:hidden">
          <span className="flex items-center gap-2">
            <ChevronDown className="w-3.5 h-3.5 transition-transform duration-200 group-data-[state=closed]:rotate-[-90deg]" />
            模型参数
          </span>
        </AccordionTrigger>
        <AccordionContent>
          <div className="space-y-3 pt-1 pb-2">
            {controls.map((control) => (
              <ModelParamControl
                key={control.name}
                control={control}
                onChange={handleParamChange}
              />
            ))}
          </div>
        </AccordionContent>
      </AccordionItem>
    </Accordion>
  );
}
