#!/usr/bin/env node

import { getSupportedFormats } from "@schema-pathfinder/core";

const formats = getSupportedFormats().join(", ");

console.log(`schema-pathfinder scaffold ready. Supported formats: ${formats}`);
