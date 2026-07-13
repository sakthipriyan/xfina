import re

with open("web/src/App.vue", "r") as f:
    content = f.read()

old_grid_regex = r'<!-- Transaction Summary Box -->.*?<!-- Market Value Box -->.*?</div>\s*</div>\s*</div>\s*<template #icon>'

new_grid = """<!-- Transaction Summary Box -->
                     <div class="border rounded-md p-3.5 bg-muted/20 flex flex-col space-y-3">
                       <span class="text-[11px] text-muted-foreground font-semibold uppercase tracking-wider flex items-center gap-1.5">
                         <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 12V7H5a2 2 0 0 1 0-4h14v4"/><path d="M3 5v14a2 2 0 0 0 2 2h16v-5"/><path d="M18 12a2 2 0 0 0 0 4h4v-4Z"/></svg>
                         Transaction Summary
                       </span>
                       <div class="space-y-2 mt-1">
                         <div class="grid grid-cols-4 gap-2 text-[10px] text-muted-foreground border-b border-border pb-1">
                           <div class="col-span-2"></div>
                           <div class="text-right">Units</div>
                           <div class="text-right">Value</div>
                         </div>
                         
                         <div class="grid grid-cols-4 gap-2 items-center pt-1">
                           <div class="col-span-2 text-[11px] text-muted-foreground">Opening Balance</div>
                           <div class="font-medium font-mono text-sm text-right">{{ formatNumber(holding.xfina?.openingBalance || 0) }}</div>
                           <div class="text-right text-muted-foreground text-[10px]">-</div>
                         </div>
                         
                         <div class="grid grid-cols-4 gap-2 items-center">
                           <div class="col-span-2 flex items-center gap-2">
                             <span class="text-[11px] text-muted-foreground">Buys</span>
                             <span class="text-[9px] bg-primary/10 text-primary px-1 rounded" v-if="holding.xfina?.periodBuyCount">{{ holding.xfina?.periodBuyCount }}</span>
                           </div>
                           <div class="font-medium font-mono text-sm text-right text-emerald-500"><span v-if="holding.xfina?.periodBuyUnits">+</span>{{ formatNumber(holding.xfina?.periodBuyUnits || 0) }}</div>
                           <div class="font-medium font-mono text-sm text-right">{{ formatCurrency(holding.xfina?.periodInvestedValue || 0) }}</div>
                         </div>
                         
                         <div class="grid grid-cols-4 gap-2 items-center">
                           <div class="col-span-2 flex items-center gap-2">
                             <span class="text-[11px] text-muted-foreground">Sells</span>
                             <span class="text-[9px] bg-primary/10 text-primary px-1 rounded" v-if="holding.xfina?.periodSellCount">{{ holding.xfina?.periodSellCount }}</span>
                           </div>
                           <div class="font-medium font-mono text-sm text-right text-rose-500"><span v-if="holding.xfina?.periodSellUnits">-</span>{{ formatNumber(holding.xfina?.periodSellUnits || 0) }}</div>
                           <div class="font-medium font-mono text-sm text-right">{{ formatCurrency(holding.xfina?.periodRealizedValue || 0) }}</div>
                         </div>
                         
                         <div class="grid grid-cols-4 gap-2 items-center pt-2 border-t border-border">
                           <div class="col-span-2 text-xs font-medium text-foreground">Closing Balance</div>
                           <div class="font-bold font-mono text-sm text-primary text-right">{{ formatNumber(holding.units) }}</div>
                           <div class="text-right text-muted-foreground text-[10px]">-</div>
                         </div>
                       </div>
                     </div>
                     
                     <!-- Market Value Box -->
                     <div class="border rounded-md p-3.5 bg-muted/20 flex flex-col space-y-3">
                       <span class="text-[11px] text-muted-foreground font-semibold uppercase tracking-wider flex items-center gap-1.5">
                         <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="12" x2="12" y1="20" y2="10"/><line x1="18" x2="18" y1="20" y2="4"/><line x1="6" x2="6" y1="20" y2="16"/></svg>
                         Holding
                       </span>
                       <div class="space-y-3 mt-2">
                         <div class="flex justify-between items-center">
                           <span class="text-[11px] text-muted-foreground">Cost Value</span>
                           <span class="font-medium font-mono text-sm">{{ formatCurrency((holding.units || 0) * (holding.rate || 0)) }}</span>
                         </div>
                         
                         <div class="flex justify-between items-center">
                           <span class="text-[11px] text-muted-foreground">Market Value</span>
                           <span class="font-bold font-mono text-sm text-primary">{{ formatCurrency((holding.units || 0) * (holding.lastTradedPrice || 0)) }}</span>
                         </div>
                         
                         <div class="flex justify-between items-center">
                           <span class="text-[11px] text-muted-foreground">Total Units</span>
                           <span class="font-medium font-mono text-sm">{{ formatNumber(holding.units) }}</span>
                         </div>
                         
                         <div class="flex justify-between items-end border-t border-border pt-3 mt-1">
                           <div class="flex flex-col">
                             <span class="text-[11px] text-foreground font-medium">NAV</span>
                             <div class="flex items-baseline gap-1 mt-0.5 text-[10px] text-muted-foreground" v-if="equityStatement.xfina?.generatedDate">
                               <span>as on</span>
                               <span class="font-medium font-mono">{{ formatDateTime(equityStatement.xfina?.generatedDate, 'xfina.generatedDate', equityStatement.xfina?.dateOnlyPaths) }}</span>
                             </div>
                           </div>
                           <span class="font-bold font-mono text-sm">{{ formatCurrency(holding.lastTradedPrice) }}</span>
                         </div>
                       </div>
                     </div>
                   </div>
                 </div>
                 <template #icon>"""

content = re.sub(old_grid_regex, new_grid, content, flags=re.DOTALL)

with open("web/src/App.vue", "w") as f:
    f.write(content)
