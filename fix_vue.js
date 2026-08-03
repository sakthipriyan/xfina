const fs = require('fs');
const content = fs.readFileSync('web/src/App.vue', 'utf8');
let lines = content.split('\n');

function checkTags(start, end) {
  let depth = 0;
  for (let i = start; i <= end; i++) {
    const line = lines[i];
    if (line.includes('<div')) depth++;
    if (line.includes('</div')) depth--;
    console.log(`${i+1}: [depth: ${depth}] ${line}`);
  }
}

// MF AccordionItem is around 700 to 865
console.log("MF:");
checkTags(700, 865);
console.log("\nEquity:");
checkTags(1060, 1150);
