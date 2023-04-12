//import * as oura from "https://raw.githubusercontent.com/txpipe/oura/v2/assets/denopkgs/v2AlphaOuraUtils/mod.ts";

export async function mapEvent(event: any) {
  // do whatever you want with the original event and
  // return a new object that will be sent to the next
  // stage in the pipeline
  const event2 = { ...event, extraProp: "123abc" };

  return event2;
}
