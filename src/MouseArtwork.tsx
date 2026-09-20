import { useId } from "react";
import { mouseDrawings } from "./mouseGeometry";

type Props = {
  side?: boolean;
  selected: string;
  onSelect: (button: string) => void;
};

export default function MouseArtwork({ side = false, selected, onSelect }: Props) {
  const drawing = mouseDrawings[side ? "x1-side" : "x1"];
  const id = useId().replace(/:/g, "");
  return <svg className="mouse-map-artwork"
    viewBox={`0 0 ${drawing.width} ${drawing.height}`} preserveAspectRatio="xMidYMid meet"
    role="group" aria-label={side ? "X1 side buttons" : "X1 mouse buttons"} focusable="false">
    <defs>
      {drawing.zones.map(zone => <clipPath key={zone.button} id={id+"-"+zone.button.replace(" ", "")}><path d={zone.path}/></clipPath>)}
    </defs>
    <image href={drawing.image} width={drawing.width} height={drawing.height} pointerEvents="none"/>
    {drawing.zones.map(zone => <path key={zone.button} d={zone.path}
      className={"mouse-hit-zone"+(selected === zone.button ? " selected" : "")}
      clipPath={`url(#${id}-${zone.button.replace(" ", "")})`}
      role="button" tabIndex={0} aria-label={"Select "+zone.label} aria-pressed={selected === zone.button}
      data-button={zone.button} vectorEffect="non-scaling-stroke"
      onClick={() => onSelect(zone.button)}
      onKeyDown={event => {
        if (event.key === "Enter" || event.key === " ") { event.preventDefault(); onSelect(zone.button); }
      }}/>
    )}
  </svg>;
}
