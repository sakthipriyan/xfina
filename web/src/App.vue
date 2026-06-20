<script setup>
import { ref, onMounted } from 'vue';
import { useDark, useToggle } from '@vueuse/core';
import init, { parse_ibkr } from './wasm/financial_extract_wasm.js';
import { Sun, Moon, Github, HelpCircle } from 'lucide-vue-next';

// Shadcn components
import { Button } from '@/components/ui/button';
import { Select, SelectContent, SelectGroup, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Accordion, AccordionContent, AccordionItem, AccordionTrigger } from '@/components/ui/accordion';

const isDark = useDark();
const toggleDark = () => {
    isDark.value = !isDark.value;
    if (isDark.value) {
        document.documentElement.classList.add('dark');
    } else {
        document.documentElement.classList.remove('dark');
    }
};

onMounted(() => {
    if (isDark.value) {
        document.documentElement.classList.add('dark');
    } else {
        document.documentElement.classList.remove('dark');
    }
});

const wasmLoaded = ref(false);
const error = ref(null);
const portfolio = ref(null);

const sources = ref([
    { label: 'Interactive Brokers (IBKR)', value: 'IBKR' },
    { label: 'CAMS (Mutual Funds)', value: 'CAMS' }
]);
const selectedSource = ref('IBKR');

onMounted(async () => {
    try {
        await init();
        wasmLoaded.value = true;
    } catch (e) {
        error.value = "Failed to load WebAssembly module: " + e;
    }
});

const onFileSelect = async (event) => {
    const file = event.target.files[0];
    if (!file) return;

    if (selectedSource.value !== 'IBKR') {
        error.value = "Only IBKR is currently supported in this WebAssembly PoC.";
        return;
    }

    error.value = null;
    portfolio.value = null;
    try {
        const text = await file.text();
        
        const start = performance.now();
        const jsonString = parse_ibkr(text);
        const end = performance.now();
        
        console.log(`🚀 Rust WASM Processing Time: ${(end - start).toFixed(2)} ms`);
        
        portfolio.value = JSON.parse(jsonString);
    } catch (e) {
        error.value = "Error parsing file: " + e;
    }
};

const getCurrencySymbol = () => {
    if (selectedSource.value === 'IBKR') {
        return '$';
    }
    return ''; // Can default to ₹ later
};

const formatCurrency = (val) => {
    if (val === null || val === undefined) return '-';
    return getCurrencySymbol() + Number(val).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
};

const formatNumber = (val) => {
    if (val === null || val === undefined) return '-';
    return Number(val).toLocaleString(undefined, { minimumFractionDigits: 0, maximumFractionDigits: 4 });
};

const formatDateLocal = (dateStr) => {
    if (!dateStr) return '-';
    
    // Check if it's a date only YYYY-MM-DD
    if (/^\d{4}-\d{2}-\d{2}$/.test(dateStr)) {
        const d = new Date(dateStr + "T00:00:00");
        if (isNaN(d)) return dateStr;
        return new Intl.DateTimeFormat(undefined, { 
            year: 'numeric', 
            month: 'short', 
            day: 'numeric' 
        }).format(d);
    }

    const d = new Date(dateStr);
    if (isNaN(d)) return dateStr;

    return new Intl.DateTimeFormat(undefined, { 
        year: 'numeric', 
        month: 'short', 
        day: 'numeric',
        hour: '2-digit', 
        minute: '2-digit', 
        second: '2-digit'
    }).format(d);
};
</script>

<template>
  <div class="min-h-screen bg-background text-foreground p-8 font-sans transition-colors duration-200">
    <div class="max-w-6xl mx-auto space-y-8">
      
      <!-- Header -->
      <div class="flex flex-col md:flex-row md:justify-between md:items-center gap-4">
        <div class="space-y-2">
          <h1 class="text-3xl font-bold tracking-tight">extract.sakthipriyan.com</h1>
          <p class="text-muted-foreground mt-2 leading-relaxed">
            Parse financial statements entirely in your browser with Rust/Wasm<br />
            Fast, private, zero-setup, and without uploading your files to any server.
          </p>
        </div>
        <div class="flex items-center space-x-3">
          <a href="https://github.com/sakthipriyan/financial-extract" target="_blank" rel="noopener noreferrer" class="no-underline">
            <Button variant="outline" class="flex items-center gap-2 px-3">
              <Github class="h-[1.2rem] w-[1.2rem]" />
              <span class="font-medium">Open Source</span>
            </Button>
          </a>
          <Button variant="outline" size="icon" @click="toggleDark()">
            <Sun v-if="isDark" class="h-[1.2rem] w-[1.2rem] text-foreground" />
            <Moon v-else class="h-[1.2rem] w-[1.2rem] text-foreground" />
            <span class="sr-only">Toggle theme</span>
          </Button>
        </div>
      </div>
      
      <!-- Error Message -->
      <div v-if="error" class="p-4 bg-destructive/10 border border-destructive/20 rounded-md text-destructive">
        {{ error }}
      </div>
      
      <!-- Upload Zone -->
      <Card v-if="wasmLoaded" class="bg-card border-border shadow-sm">
        <CardHeader>
          <CardTitle>Parse Statement</CardTitle>
          <CardDescription>Upload a CSV or PDF statement to extract your portfolio data directly in the browser.</CardDescription>
        </CardHeader>
        <CardContent>
          <div class="flex flex-col md:flex-row gap-6 items-end">
            <div class="space-y-2 w-full md:w-64">
               <Label>Source Broker</Label>
               <Select v-model="selectedSource">
                 <SelectTrigger class="w-full bg-background border-border">
                   <SelectValue placeholder="Select Source" />
                 </SelectTrigger>
                 <SelectContent class="bg-card border-border">
                   <SelectGroup>
                     <SelectItem v-for="src in sources" :key="src.value" :value="src.value">
                       {{ src.label }}
                     </SelectItem>
                   </SelectGroup>
                 </SelectContent>
               </Select>
            </div>
            <div class="space-y-2 w-full flex-1">
               <Label>Upload File</Label>
               <Input 
                  type="file" 
                  accept=".csv" 
                  @change="onFileSelect" 
                  class="cursor-pointer bg-background border-border text-foreground file:bg-secondary file:text-secondary-foreground file:border-0 file:mr-4 file:px-4 file:py-2 file:rounded hover:file:bg-secondary/80 transition-colors" 
                />
            </div>
          </div>
        </CardContent>
      </Card>
      <div v-else class="text-muted-foreground animate-pulse">Loading WebAssembly module...</div>
      
      <!-- Results Table -->
      <div v-if="portfolio" class="space-y-6">
        
        <!-- Investor Info Card -->
        <Card v-if="portfolio.investor_info">
          <CardHeader>
            <CardTitle class="flex flex-col sm:flex-row sm:justify-between sm:items-center gap-2">
              <div class="flex items-center gap-3">
                <span class="text-xl">{{ portfolio.investor_info.name || 'Investor' }}</span>
                <span v-if="portfolio.investor_info.account_number" class="text-sm font-mono text-muted-foreground bg-muted px-2 py-0.5 rounded">{{ portfolio.investor_info.account_number }}</span>
              </div>
            </CardTitle>
            <CardDescription class="flex flex-col sm:flex-row gap-2 sm:gap-4 mt-1">
              <span v-if="portfolio.statement_start_date && portfolio.statement_end_date">Period: {{ formatDateLocal(portfolio.statement_start_date) }} to {{ formatDateLocal(portfolio.statement_end_date) }}</span>
              <span v-if="portfolio.generated_date">Generated: {{ formatDateLocal(portfolio.generated_date) }}</span>
            </CardDescription>
          </CardHeader>
        </Card>

        <Accordion type="multiple" class="w-full space-y-4">
           <AccordionItem 
             v-for="(asset, index) in portfolio.assets" 
             :key="index" 
             :value="`item-${index}`"
             class="border rounded-lg bg-card text-card-foreground shadow-sm overflow-hidden"
           >
               <AccordionTrigger class="hover:no-underline px-4 py-4 data-[state=open]:border-b border-border">
                 <div class="flex flex-col w-full text-left pr-4 space-y-3">
                   <div class="flex justify-between items-start w-full">
                     <div class="flex flex-col items-start">
                       <span class="font-medium text-foreground text-lg">{{ asset.name }}</span>
                       <span class="text-sm text-muted-foreground mt-0.5">{{ asset.symbol || '-' }} <span v-if="asset.isin">| ISIN: {{ asset.isin }}</span></span>
                     </div>
                     <div class="text-xs font-mono bg-primary/10 text-primary px-2 py-1 rounded">
                       {{ asset.transactions ? asset.transactions.length : 0 }} Txns
                     </div>
                   </div>
                   
                   <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-5 gap-4">
                     <!-- Period Activity -->
                     <div class="flex flex-col border-r pr-4">
                       <span class="text-xs text-muted-foreground font-semibold mb-1 uppercase tracking-wider">Period Activity</span>
                       <div class="flex flex-col mt-1">
                         <span class="text-muted-foreground text-xs">Units Added</span>
                         <span class="font-medium font-mono">{{ formatNumber(asset.period_units) }}</span>
                       </div>
                       <div class="flex flex-col mt-2">
                         <span class="text-muted-foreground text-xs">Invested</span>
                         <span class="font-medium font-mono">{{ formatCurrency(asset.period_invested_value) }}</span>
                       </div>
                       <div v-if="asset.period_realized_value > 0" class="flex flex-col mt-2">
                         <span class="text-muted-foreground text-xs">Realized / Sold</span>
                         <span class="font-medium font-mono">{{ formatCurrency(asset.period_realized_value) }}</span>
                       </div>
                     </div>
                     
                     <!-- Overall Balance -->
                     <div class="flex flex-col col-span-1 md:col-span-2 lg:col-span-4 pl-0 md:pl-2">
                       <span class="text-xs text-muted-foreground font-semibold mb-1 uppercase tracking-wider">Overall Balance</span>
                       <div class="grid grid-cols-2 lg:grid-cols-4 gap-4 mt-1">
                         <div class="flex flex-col">
                           <span class="text-muted-foreground text-xs flex items-center gap-1 cursor-help" title="Total Cost Basis">
                             Invested
                             <HelpCircle class="w-3 h-3 text-muted-foreground/70" />
                           </span>
                           <span class="font-medium font-mono">{{ formatCurrency(asset.total_cost_basis) }}</span>
                         </div>
                         <div class="flex flex-col">
                           <span class="text-muted-foreground text-xs">Total Units</span>
                           <span class="font-medium font-mono">{{ formatNumber(asset.total_units) }}</span>
                         </div>
                         <div class="flex flex-col">
                           <span class="text-muted-foreground text-xs">NAV / Price</span>
                           <div class="flex flex-col">
                             <span class="font-medium font-mono">{{ formatCurrency(asset.current_nav) }}</span>
                             <span v-if="asset.current_nav_date" class="font-medium font-mono">on {{ formatDateLocal(asset.current_nav_date) }}</span>
                           </div>
                         </div>
                         <div class="flex flex-col">
                           <span class="text-muted-foreground text-xs">Market Value</span>
                           <span class="font-medium font-mono text-primary">{{ formatCurrency(asset.current_value) }}</span>
                         </div>
                       </div>
                     </div>
                   </div>
                 </div>
               </AccordionTrigger>
               <AccordionContent>
                 <div class="rounded-md border border-border mt-2 overflow-x-auto">
                   <Table>
                     <TableHeader class="bg-muted/50">
                       <TableRow class="hover:bg-transparent">
                         <TableHead class="text-muted-foreground whitespace-nowrap">Date</TableHead>
                         <TableHead class="text-muted-foreground whitespace-nowrap">Description / Type</TableHead>
                         <TableHead class="text-right text-muted-foreground whitespace-nowrap">Total Amount</TableHead>
                         <TableHead class="text-right text-muted-foreground whitespace-nowrap">Units / Qty</TableHead>
                         <TableHead class="text-right text-muted-foreground whitespace-nowrap">NAV / Price</TableHead>
                         <TableHead class="text-right text-muted-foreground whitespace-nowrap">Duty / STT / Fee</TableHead>
                         <TableHead class="text-right text-muted-foreground whitespace-nowrap">Balance</TableHead>
                       </TableRow>
                     </TableHeader>
                     <TableBody>
                       <TableRow v-for="(txn, idx) in asset.transactions" :key="idx" class="hover:bg-muted/50 transition-colors">
                         <TableCell class="text-foreground whitespace-nowrap">{{ formatDateLocal(txn.date) }}</TableCell>
                         <TableCell class="text-foreground">{{ txn.tx_type || '-' }}</TableCell>
                         <TableCell class="text-right font-mono text-foreground">{{ formatCurrency(txn.amount) }}</TableCell>
                         <TableCell class="text-right font-mono text-foreground">{{ formatNumber(txn.units) }}</TableCell>
                         <TableCell class="text-right font-mono text-foreground">{{ formatCurrency(txn.nav) }}</TableCell>
                         <TableCell class="text-right font-mono text-foreground">{{ txn.fee ? formatCurrency(Math.abs(txn.fee)) : '-' }}</TableCell>
                         <TableCell class="text-right font-mono text-foreground">{{ formatNumber(txn.balance) }}</TableCell>
                       </TableRow>
                     </TableBody>
                   </Table>
                 </div>
               </AccordionContent>
             </AccordionItem>
            </Accordion>
      </div>
      
    </div>
  </div>
</template>

<style>
</style>
