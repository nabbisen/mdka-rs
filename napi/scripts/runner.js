#!/usr/bin/env node

// examples which will not be delivered

const napiRun = require("../index.js");

napiRun.fromHtml("<h1>Hello, world.</h1>")
napiRun.fromFile("./awesome.html")
napiRun.fromHtmlToFile("<h1>Hello, world.</h1>", "./awesome.md", false)
napiRun.fromFileToFile("./awesome.html", "./awesome.md", false)
