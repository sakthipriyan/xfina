<script setup>
import { ref, onMounted } from 'vue';
import { useDark, useToggle } from '@vueuse/core';
import init, { parse_ibkr } from './wasm/financial_extract_wasm.js';
import { Sun, Moon, Github } from 'lucide-vue-next';

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

const formatCurrency = (val) => {
    if (val === null || val === undefined) return '-';
    return Number(val).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
};

const formatNumber = (val) => {
    if (val === null || val === undefined) return '-';
    return Number(val).toLocaleString(undefined, { minimumFractionDigits: 0, maximumFractionDigits: 4 });
};
</script>

<template>
  <div class="min-h-screen bg-background text-foreground p-8 font-sans transition-colors duration-200">
    <div class="max-w-6xl mx-auto space-y-8">
      
      <!-- Header -->
      <div class="flex justify-between items-center">
        <div class="space-y-2">
          <h1 class="text-3xl font-bold tracking-tight">extract.sakthipriyan.com</h1>
          <p class="text-muted-foreground">High-performance Rust WebAssembly parser for broker statements</p>
        </div>
        <div class="flex items-center space-x-2">
          <a href="https://github.com/sakthipriyan/financial-extract" target="_blank" rel="noopener noreferrer">
            <Button variant="outline" size="icon">
              <Github class="h-[1.2rem] w-[1.2rem] text-foreground" />
              <span class="sr-only">GitHub Repo</span>
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
      <Card v-if="portfolio && portfolio.assets && portfolio.assets.length > 0" class="bg-card border-border shadow-md">
        <CardHeader>
          <CardTitle>Extracted Portfolio</CardTitle>
          <CardDescription>Successfully parsed {{ portfolio.assets.length }} assets.</CardDescription>
        </CardHeader>
        <CardContent>
           <Accordion type="multiple" class="w-full space-y-4">
             <AccordionItem v-for="asset in portfolio.assets" :key="asset.symbol || asset.name" :value="asset.symbol || asset.name" class="border border-border rounded-md px-4 bg-background">
               <AccordionTrigger class="hover:no-underline">
                 <div class="flex justify-between items-center w-full pr-4">
                   <div class="flex flex-col items-start text-left">
                     <span class="font-medium text-foreground">{{ asset.name }}</span>
                     <span class="text-sm text-muted-foreground">{{ asset.symbol || '-' }} <span v-if="asset.isin">| ISIN: {{ asset.isin }}</span></span>
                   </div>
                   <div class="text-sm font-mono text-muted-foreground">
                     {{ asset.transactions ? asset.transactions.length : 0 }} Txns
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
                         <TableHead class="text-right text-muted-foreground whitespace-nowrap">Amount</TableHead>
                         <TableHead class="text-right text-muted-foreground whitespace-nowrap">Units / Qty</TableHead>
                         <TableHead class="text-right text-muted-foreground whitespace-nowrap">NAV / Price</TableHead>
                         <TableHead class="text-right text-muted-foreground whitespace-nowrap">Duty / STT / Fee</TableHead>
                         <TableHead class="text-right text-muted-foreground whitespace-nowrap">Balance</TableHead>
                       </TableRow>
                     </TableHeader>
                     <TableBody>
                       <TableRow v-for="(txn, idx) in asset.transactions" :key="idx" class="hover:bg-muted/50 transition-colors">
                         <TableCell class="text-foreground whitespace-nowrap">{{ txn.date || '-' }}</TableCell>
                         <TableCell class="text-foreground">{{ txn.tx_type || '-' }}</TableCell>
                         <TableCell class="text-right font-mono text-foreground">{{ formatCurrency(txn.amount) }}</TableCell>
                         <TableCell class="text-right font-mono text-foreground">{{ formatNumber(txn.units) }}</TableCell>
                         <TableCell class="text-right font-mono text-foreground">{{ formatCurrency(txn.nav) }}</TableCell>
                         <TableCell class="text-right font-mono text-destructive text-sm">{{ formatCurrency(txn.fee) }}</TableCell>
                         <TableCell class="text-right font-mono text-foreground">{{ formatNumber(txn.balance) }}</TableCell>
                       </TableRow>
                     </TableBody>
                   </Table>
                 </div>
               </AccordionContent>
             </AccordionItem>
           </Accordion>
        </CardContent>
      </Card>
      
    </div>
  </div>
</template>

<style>
</style>
