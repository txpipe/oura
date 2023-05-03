import { Sheets } from "./deps.ts";

const sheets = new Sheets();
const sheet = await sheets.spreadsheetsCreate({});

console.log("pepe");
