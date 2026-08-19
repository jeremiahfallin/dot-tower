#!/bin/sh
# Inlines model.mjs into template.html -> progression-curve.html (single file, double-clickable).
cd "$(dirname "$0")"
node -e "
const fs=require('fs');
const model=fs.readFileSync('model.mjs','utf8').replace(/^export /gm,'');
const tpl=fs.readFileSync('template.html','utf8');
fs.writeFileSync('progression-curve.html', tpl.replace('/*__MODEL__*/', model));
console.log('built progression-curve.html');
"
