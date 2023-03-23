import * as hex from "https://deno.land/std/encoding/hex.ts";
import * as oura from "./ouraTypes.ts";

export type PlutusMap = Array<{
  k: { bytes: string };
  v: { bytes: string };
}>;

const TEXT_ENCODER = new TextEncoder();
const TEXT_DECODER = new TextDecoder();

function isReadableAscii(raw: string): boolean {
  return raw.split("").every((char) => {
    const charCode = char.charCodeAt(0);
    return 0x20 <= charCode && charCode <= 0x7e;
  });
}

function hexToText(hexString: string): string {
  const hexBytes = TEXT_ENCODER.encode(hexString);
  const utfBytes = hex.decode(hexBytes);
  return TEXT_DECODER.decode(utfBytes);
}

export function plutusMapToPlainJson(source: PlutusMap): oura.GenericJson {
  return source.reduce<oura.GenericJson>((all, item) => {
    const key = hexToText(item.k.bytes);
    const maybeText = hexToText(item.v.bytes);
    all[key] = isReadableAscii(maybeText) ? maybeText : item.v.bytes;
    return all;
  }, {});
}
