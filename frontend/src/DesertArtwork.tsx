import { DESERT_ART, type DesertKind } from "./desertTiles";

/**
 * The stylised oasis/mirage faces, shared by placed tiles, dialog previews and
 * cockpit icons. Drawn in a 100 x 100 local square; the caller supplies the SVG
 * (or group) and its size.
 */
export default function DesertArtwork({ kind }: { readonly kind: DesertKind }) {
  return kind === "oasis" ? (
    <g>
      <path d={DESERT_ART.oasis.water} className="desert-oasis-water" />
      <path d={DESERT_ART.oasis.trunk} className="desert-oasis-trunk" />
      <path d={DESERT_ART.oasis.fronds} className="desert-oasis-fronds" />
    </g>
  ) : (
    <g>
      <path d={DESERT_ART.mirage.dune} className="desert-mirage-dune" />
      <path d={DESERT_ART.mirage.haze} className="desert-mirage-haze" />
    </g>
  );
}
