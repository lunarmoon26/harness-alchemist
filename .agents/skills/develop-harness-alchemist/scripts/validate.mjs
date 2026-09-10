#!/usr/bin/env node

import { runValidate } from "../../../../lib/validate.mjs"

process.exitCode = await runValidate(process.argv.slice(2))
