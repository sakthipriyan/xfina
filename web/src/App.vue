<script setup>
import { ref, onMounted } from 'vue';
import init, { parse_ibkr } from './wasm/financial_extract_wasm.js';

// Shadcn components
import { Button } from '@/components/ui/button';
import { Select, SelectContent, SelectGroup, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';

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
</script>

<template>
  <div class="min-h-screen bg-zinc-950 text-zinc-50 p-8 font-sans">
    <div class="max-w-5xl mx-auto space-y-8">
      
      <!-- Header -->
      <div class="space-y-2">
        <h1 class="text-3xl font-bold tracking-tight">financial-extract</h1>
        <p class="text-zinc-400">High-performance Rust WebAssembly parser for broker statements</p>
      </div>
      
      <!-- Error Message -->
      <div v-if="error" class="p-4 bg-red-950/50 border border-red-900 rounded-md text-red-200">
        {{ error }}
      </div>
      
      <!-- Upload Zone -->
      <Card v-if="wasmLoaded" class="bg-zinc-900 border-zinc-800">
        <CardHeader>
          <CardTitle>Parse Statement</CardTitle>
          <CardDescription>Upload a CSV or PDF statement to extract your portfolio data directly in the browser.</CardDescription>
        </CardHeader>
        <CardContent>
          <div class="flex flex-col md:flex-row gap-6 items-end">
            <div class="space-y-2 w-full md:w-64">
               <Label>Source Broker</Label>
               <Select v-model="selectedSource">
                 <SelectTrigger class="w-full bg-zinc-950 border-zinc-800">
                   <SelectValue placeholder="Select Source" />
                 </SelectTrigger>
                 <SelectContent class="bg-zinc-900 border-zinc-800">
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
                  class="cursor-pointer bg-zinc-950 border-zinc-800 text-zinc-300 file:bg-zinc-800 file:text-zinc-50 file:border-0 file:mr-4 file:px-4 file:py-2 file:rounded hover:file:bg-zinc-700 transition-colors" 
                />
            </div>
          </div>
        </CardContent>
      </Card>
      <div v-else class="text-zinc-500 animate-pulse">Loading WebAssembly module...</div>
      
      <!-- Results Table -->
      <Card v-if="portfolio && portfolio.assets && portfolio.assets.length > 0" class="bg-zinc-900 border-zinc-800 shadow-xl">
        <CardHeader>
          <CardTitle>Extracted Portfolio</CardTitle>
          <CardDescription>Successfully parsed {{ portfolio.assets.length }} assets in milliseconds.</CardDescription>
        </CardHeader>
        <CardContent>
           <div class="rounded-md border border-zinc-800 overflow-hidden">
             <Table>
               <TableHeader class="bg-zinc-950/50">
                 <TableRow class="border-zinc-800 hover:bg-transparent">
                   <TableHead class="text-zinc-400">Symbol</TableHead>
                   <TableHead class="text-zinc-400">Name</TableHead>
                   <TableHead class="text-zinc-400">ISIN</TableHead>
                   <TableHead class="text-right text-zinc-400">Transactions</TableHead>
                 </TableRow>
               </TableHeader>
               <TableBody>
                 <TableRow v-for="asset in portfolio.assets" :key="asset.symbol || asset.name" class="border-zinc-800/50 hover:bg-zinc-800/50 transition-colors">
                   <TableCell class="font-medium text-zinc-300">{{ asset.symbol || '-' }}</TableCell>
                   <TableCell>{{ asset.name }}</TableCell>
                   <TableCell class="text-zinc-500">{{ asset.isin || '-' }}</TableCell>
                   <TableCell class="text-right font-mono">{{ asset.transactions ? asset.transactions.length : 0 }}</TableCell>
                 </TableRow>
               </TableBody>
             </Table>
           </div>
        </CardContent>
      </Card>
      
    </div>
  </div>
</template>

<style>
/* Remove PrimeVue resets if any, and rely purely on Tailwind */
</style>
