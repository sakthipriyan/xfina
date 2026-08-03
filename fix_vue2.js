const fs = require('fs');
const content = fs.readFileSync('web/src/App.vue', 'utf8');
const lines = content.split('\n');

function checkTags(start, end) {
  let stack = [];
  for (let i = start; i <= end; i++) {
    const line = lines[i];
    // Simple regex to find open and close tags
    const openTags = [...line.matchAll(/<([a-zA-Z0-9]+)(?![^>]*\/>)[^>]*>/g)].map(m => m[1]);
    const closeTags = [...line.matchAll(/<\/([a-zA-Z0-9]+)>/g)].map(m => m[1]);
    
    for (const tag of openTags) {
      // Ignore self-closing tags that might not have /> like img, br, input, hr (but Vue requires them or we can just ignore known voids)
      if (['img', 'br', 'hr', 'input'].includes(tag.toLowerCase())) continue;
      stack.push({tag, line: i+1});
    }
    
    for (const tag of closeTags) {
      if (stack.length > 0 && stack[stack.length-1].tag === tag) {
        stack.pop();
      } else {
        console.log(`Mismatch at line ${i+1}: expected ${stack.length ? stack[stack.length-1].tag : 'nothing'}, found ${tag}`);
      }
    }
  }
  console.log("Remaining in stack:");
  for (const item of stack) {
    console.log(`Line ${item.line}: <${item.tag}>`);
  }
}

console.log("MF:");
checkTags(700, 865);
console.log("\nEquity:");
checkTags(1060, 1152);
