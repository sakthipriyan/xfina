<script setup>
import { ref, onMounted } from 'vue';
import { useDark, useToggle } from '@vueuse/core';
import init, { parse_ibkr, parse_cams, parse_hdfc_cc, parse_icici_cc } from './wasm/finx_wasm.js';
import { Sun, Moon, Github, HelpCircle, ChevronDown, Loader2 } from 'lucide-vue-next';

// Shadcn components
import { Button } from '@/components/ui/button';
import { Select, SelectContent, SelectGroup, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';
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
const ccStatement = ref(null);
const isProcessing = ref(false);
const parseTime = ref(null);

const selectedCategory = ref('Mutual Funds');
const selectedSource = ref('CAMS');
const password = ref('');

const setCategory = (cat) => {
    selectedCategory.value = cat;
    portfolio.value = null;
    ccStatement.value = null;
    error.value = null;
    if (cat === 'Mutual Funds') selectedSource.value = 'CAMS';
    else if (cat === 'Intl Stocks') selectedSource.value = 'IBKR';
    else if (cat === 'Credit Cards') selectedSource.value = 'HDFC';
};

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

    error.value = null;
    portfolio.value = null;
    ccStatement.value = null;
    isProcessing.value = true;
    parseTime.value = null;
    
    // Yield to the event loop so the "Parsing..." UI can render
    await new Promise(resolve => setTimeout(resolve, 10));

    try {
        let jsonString;
        const start = performance.now();
        
        if (selectedSource.value === 'IBKR') {
            const text = await file.text();
            jsonString = parse_ibkr(text);
            portfolio.value = JSON.parse(jsonString);
        } else if (selectedSource.value === 'CAMS') {
            const arrayBuffer = await file.arrayBuffer();
            const uint8Array = new Uint8Array(arrayBuffer);
            jsonString = parse_cams(uint8Array, password.value ? password.value : null);
            portfolio.value = JSON.parse(jsonString);
        } else if (selectedSource.value === 'HDFC') {
            const text = await file.text();
            jsonString = parse_hdfc_cc(text);
            ccStatement.value = JSON.parse(jsonString);
        } else if (selectedSource.value === 'ICICI') {
            const arrayBuffer = await file.arrayBuffer();
            const uint8Array = new Uint8Array(arrayBuffer);
            jsonString = parse_icici_cc(uint8Array);
            ccStatement.value = JSON.parse(jsonString);
        }
        
        const end = performance.now();
        parseTime.value = ((end - start) / 1000).toFixed(3);
        console.log(`🚀 Rust WASM Processing Time: ${(end - start).toFixed(2)} ms`);

    } catch (e) {
        error.value = "Error parsing file: " + e;
    } finally {
        isProcessing.value = false;
    }
};

const getCurrencySymbol = () => {
    if (selectedSource.value === 'IBKR') {
        return '$';
    }
    return '₹'; // Default to Rupee for CAMS
};

const formatCurrency = (val) => {
    if (val === null || val === undefined) return '-';
    const num = Number(val);
    const formatted = Math.abs(num).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
    return (num < 0 ? '-' : '') + getCurrencySymbol() + formatted;
};

const formatNumber = (val) => {
    if (val === null || val === undefined) return '-';
    return Number(val).toLocaleString(undefined, { minimumFractionDigits: 0, maximumFractionDigits: 4 });
};

const formatDateLocal = (dateStr) => {
    if (!dateStr) return '-';
    
    let parseStr = dateStr.trim();
    
    if (/^\d{4}-\d{2}-\d{2}$/.test(parseStr)) {
        parseStr = parseStr + "T00:00:00";
    }

    const d = new Date(parseStr);
    if (isNaN(d)) return dateStr;

    const hasTime = dateStr.includes(':') || (dateStr.includes('T') && !dateStr.endsWith('T00:00:00.000Z') && !dateStr.endsWith('T00:00:00Z') && !dateStr.endsWith('T00:00:00'));

    if (hasTime) {
        return new Intl.DateTimeFormat(undefined, { 
            year: 'numeric', 
            month: 'short', 
            day: 'numeric',
            hour: '2-digit', 
            minute: '2-digit', 
            second: '2-digit'
        }).format(d);
    } else {
        return new Intl.DateTimeFormat(undefined, { 
            year: 'numeric', 
            month: 'short', 
            day: 'numeric' 
        }).format(d);
    }
};

const hasRewards = (stmt) => {
    if (!stmt?.reward_points_summary) return false;
    const s = stmt.reward_points_summary;
    return s.opening_balance !== 0 || 
           s.earned !== 0 || 
           s.disbursed !== 0 || 
           s.adjusted_lapsed !== 0 || 
           s.closing_balance !== 0 || 
           s.default_rewards !== 0;
};
</script>

<template>
  <div class="min-h-screen bg-background text-foreground p-8 font-sans transition-colors duration-200">
    <div class="max-w-6xl mx-auto space-y-8">
      
      <!-- Header -->
      <div class="flex flex-col md:flex-row md:justify-between md:items-center gap-4">
        <div class="space-y-2">
          <h1 class="text-3xl font-bold tracking-tight">finx.sakthipriyan.com</h1>
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
        <CardHeader class="flex flex-row items-start justify-between space-y-0 pb-4">
          <div class="space-y-1.5">
            <CardTitle>Parse Statement</CardTitle>
            <CardDescription>Upload your statement to securely extract and view your financial data directly in the browser.</CardDescription>
          </div>
          <div v-if="isProcessing" class="flex items-center text-sm font-medium text-muted-foreground gap-2 whitespace-nowrap mt-0.5">
            <span>Parsing...</span>
            <Loader2 class="h-4 w-4 animate-spin" />
          </div>
          <div v-else-if="parseTime !== null" class="text-sm font-medium text-muted-foreground whitespace-nowrap mt-0.5">
            Parsed in {{ parseTime }}s
          </div>
        </CardHeader>
        <CardContent>
          <div class="flex flex-wrap gap-2 mb-6">
            <Button 
              :variant="selectedCategory === 'Mutual Funds' ? 'default' : 'outline'"
              @click="setCategory('Mutual Funds')"
            >Mutual Funds</Button>
            <Button 
              :variant="selectedCategory === 'Intl Stocks' ? 'default' : 'outline'"
              @click="setCategory('Intl Stocks')"
            >Intl Stocks</Button>
            <Button 
              :variant="selectedCategory === 'Credit Cards' ? 'default' : 'outline'"
              @click="setCategory('Credit Cards')"
            >Credit Cards</Button>
          </div>

          <div class="flex flex-col md:flex-row gap-6 items-end">
             <div class="space-y-2 w-full md:w-64" v-if="selectedCategory === 'Mutual Funds'">
               <Label>Provider</Label>
               <Select v-model="selectedSource">
                 <SelectTrigger class="w-full bg-background border-border">
                   <SelectValue placeholder="Select Provider" />
                 </SelectTrigger>
                 <SelectContent>
                   <SelectGroup>
                     <SelectItem value="CAMS">CAMS Combined Statement</SelectItem>
                   </SelectGroup>
                 </SelectContent>
               </Select>
             </div>
             <div class="space-y-2 w-full md:w-64" v-if="selectedCategory === 'Mutual Funds'">
               <Label>PDF Password</Label>
               <Input 
                  type="password" 
                  v-model="password"
                  placeholder="Enter password" 
                  class="bg-background border-border"
                />
             </div>
             <div class="space-y-2 w-full md:w-64" v-if="selectedCategory === 'Intl Stocks'">
               <Label>Broker</Label>
               <Select v-model="selectedSource">
                 <SelectTrigger class="w-full bg-background border-border">
                   <SelectValue placeholder="Select Broker" />
                 </SelectTrigger>
                 <SelectContent>
                   <SelectGroup>
                     <SelectItem value="IBKR">Interactive Brokers</SelectItem>
                   </SelectGroup>
                 </SelectContent>
               </Select>
             </div>
             <div class="space-y-2 w-full md:w-64" v-if="selectedCategory === 'Credit Cards'">
               <Label>Bank</Label>
               <Select v-model="selectedSource">
                 <SelectTrigger class="w-full bg-background border-border">
                   <SelectValue placeholder="Select Bank" />
                 </SelectTrigger>
                 <SelectContent>
                   <SelectGroup>
                     <SelectItem value="HDFC">HDFC</SelectItem>
                     <SelectItem value="ICICI">ICICI</SelectItem>
                   </SelectGroup>
                 </SelectContent>
               </Select>
             </div>

             <div class="space-y-2 w-full flex-1">
               <Label class="flex items-baseline gap-2">
                 Upload File
                 <span class="text-xs text-muted-foreground font-normal">
                   (Supported: {{ selectedCategory === 'Mutual Funds' ? 'PDF' : (selectedCategory === 'Credit Cards' && selectedSource === 'ICICI' ? 'EXCEL' : 'CSV') }})
                 </span>
               </Label>
               <Input 
                  type="file" 
                  :accept="selectedCategory === 'Mutual Funds' ? '.pdf' : (selectedCategory === 'Credit Cards' && selectedSource === 'ICICI' ? '.xls,.xlsx' : '.csv')"
                  @change="onFileSelect" 
                  class="cursor-pointer bg-background border-border text-foreground file:bg-secondary file:text-secondary-foreground file:border-0 file:mr-4 file:px-4 file:py-1.5 file:rounded-md hover:file:bg-secondary/80 transition-colors h-auto pb-2 pt-2" 
                />
            </div>
          </div>
        </CardContent>
      </Card>
      <div v-else class="text-muted-foreground animate-pulse">Loading WebAssembly module...</div>
      
      <!-- Credit Card Results Table -->
      <div v-if="ccStatement" class="space-y-6">
        
        <!-- Statement Info Card -->
        <Card>
          <CardHeader>
            <CardTitle class="flex flex-col sm:flex-row sm:justify-between sm:items-center gap-2">
              <div class="flex items-center gap-3">
                <span class="text-xl">{{ ccStatement.customer_info?.name || 'Customer' }}</span>
                <span v-if="ccStatement.card_no" class="text-sm font-mono text-muted-foreground bg-muted px-2 py-0.5 rounded">{{ ccStatement.card_no.slice(-4) }}</span>
              </div>
            </CardTitle>
            <CardDescription class="flex flex-col sm:flex-row gap-2 sm:gap-4 mt-1">
              <span v-if="ccStatement.statement_start_date && ccStatement.statement_end_date">Period: {{ formatDateLocal(ccStatement.statement_start_date) }} to {{ formatDateLocal(ccStatement.statement_end_date) }}</span>
              <span v-if="ccStatement.statement_date">Statement Date: {{ formatDateLocal(ccStatement.statement_date) }}</span>
              <span v-if="ccStatement.payment_due_date">Due Date: {{ formatDateLocal(ccStatement.payment_due_date) }}</span>
            </CardDescription>
          </CardHeader>
        </Card>

        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          <Card class="bg-card text-card-foreground shadow-sm">
            <CardHeader class="pb-2">
              <CardTitle class="text-sm text-muted-foreground font-semibold uppercase tracking-wider">Account Summary</CardTitle>
            </CardHeader>
            <CardContent v-if="ccStatement.account_summary">
              <div class="space-y-2">
                <div class="flex justify-between items-center mb-2 border-b pb-2">
                  <span class="text-sm font-medium">Opening Balance</span>
                  <span class="font-bold font-mono text-lg text-primary">{{ formatCurrency(ccStatement.account_summary.opening_balance) }}</span>
                </div>
                
                <div class="flex justify-between items-center">
                  <span class="text-sm text-muted-foreground">Payment / Credit</span>
                  <span class="font-medium font-mono text-emerald-500">-{{ formatCurrency(ccStatement.account_summary.payment_credit) }}</span>
                </div>

                <div v-if="ccStatement.account_summary.owner_credit_breakdown && Object.keys(ccStatement.account_summary.owner_credit_breakdown).length > 1" class="pl-4 border-l-2 border-muted space-y-1 my-1">
                  <div v-for="(amount, owner) in ccStatement.account_summary.owner_credit_breakdown" :key="owner" class="flex justify-between items-center">
                    <span class="text-sm text-muted-foreground truncate mr-2">{{ owner }}</span>
                    <span class="font-medium font-mono text-sm text-emerald-500">-{{ formatCurrency(amount) }}</span>
                  </div>
                </div>
                
                <div class="flex justify-between items-center">
                  <span class="text-sm text-muted-foreground">Purchases / Debits</span>
                  <span class="font-medium font-mono text-rose-500">+{{ formatCurrency(ccStatement.account_summary.purchases_debits) }}</span>
                </div>
                
                <div v-if="ccStatement.account_summary.owner_debit_breakdown && Object.keys(ccStatement.account_summary.owner_debit_breakdown).length > 1" class="pl-4 border-l-2 border-muted space-y-1 my-1">
                  <div v-for="(amount, owner) in ccStatement.account_summary.owner_debit_breakdown" :key="owner" class="flex justify-between items-center">
                    <span class="text-sm text-muted-foreground truncate mr-2">{{ owner }}</span>
                    <span class="font-medium font-mono text-sm text-rose-500">+{{ formatCurrency(amount) }}</span>
                  </div>
                </div>

                <div v-if="ccStatement.account_summary.finance_charges > 0" class="flex justify-between items-center">
                  <span class="text-sm text-muted-foreground">Finance Charges</span>
                  <span class="font-medium font-mono text-rose-500">+{{ formatCurrency(ccStatement.account_summary.finance_charges) }}</span>
                </div>
                
                <div class="flex justify-between items-center mt-2 border-t pt-2">
                  <span class="text-sm font-medium">Total Dues</span>
                  <span class="font-bold font-mono text-lg text-primary">{{ formatCurrency(ccStatement.account_summary.total_dues) }}</span>
                </div>
                
                <div class="flex justify-between items-center mt-1">
                  <span class="text-xs text-muted-foreground">Min Amount Due</span>
                  <span class="font-medium font-mono text-xs">{{ formatCurrency(ccStatement.minimum_amount_due) }}</span>
                </div>
              </div>
            </CardContent>
          </Card>

          <Card class="bg-card text-card-foreground shadow-sm">
            <CardHeader class="pb-2">
              <CardTitle class="text-sm text-muted-foreground font-semibold uppercase tracking-wider">Credit Limits</CardTitle>
            </CardHeader>
            <CardContent>
              <div class="space-y-2">
                <div class="flex justify-between items-center"><span class="text-sm text-muted-foreground">Credit Limit</span><span class="font-medium font-mono">{{ formatCurrency(ccStatement.credit_limit) }}</span></div>
                <div class="flex justify-between items-center"><span class="text-sm text-muted-foreground">Available Limit</span><span class="font-medium font-mono">{{ formatCurrency(ccStatement.available_limit) }}</span></div>
                <div class="flex justify-between items-center"><span class="text-sm text-muted-foreground">Cash Limit</span><span class="font-medium font-mono">{{ formatCurrency(ccStatement.available_cash_limit) }}</span></div>
              </div>
            </CardContent>
          </Card>

          <Card v-if="hasRewards(ccStatement)" class="bg-card text-card-foreground shadow-sm">
            <CardHeader class="pb-2">
              <CardTitle class="text-sm text-muted-foreground font-semibold uppercase tracking-wider">Rewards Summary</CardTitle>
            </CardHeader>
            <CardContent>
              <div class="space-y-2">
                <div v-if="ccStatement.reward_points_summary.opening_balance !== 0 || ccStatement.reward_points_summary.closing_balance !== 0" class="flex justify-between items-center mb-2 border-b pb-2"><span class="text-sm font-medium">Opening Balance</span><span class="font-bold font-mono text-lg text-primary">{{ formatNumber(ccStatement.reward_points_summary.opening_balance) }}</span></div>
                
                <div class="flex justify-between items-center"><span class="text-sm text-muted-foreground">Earned</span><span class="font-medium font-mono text-emerald-500">+{{ formatNumber(ccStatement.reward_points_summary.earned) }}</span></div>
                
                <div v-if="ccStatement.reward_programs && ccStatement.reward_programs.length > 0" class="pl-4 border-l-2 border-muted space-y-1 my-1">
                  <div class="flex justify-between items-center">
                    <span class="text-sm text-muted-foreground truncate mr-2">Rewards</span>
                    <span class="font-medium font-mono text-sm text-emerald-500">+{{ formatNumber(ccStatement.reward_points_summary.default_rewards) }}</span>
                  </div>
                  <div v-for="(prog, idx) in ccStatement.reward_programs" :key="idx" class="flex justify-between items-center">
                    <span class="text-sm text-muted-foreground truncate mr-2" :title="prog.program">{{ prog.program }}</span>
                    <span class="font-medium font-mono text-sm text-emerald-500">+{{ formatNumber(prog.bonus_points) }}</span>
                  </div>
                </div>

                <div v-if="ccStatement.reward_points_summary.disbursed > 0" class="flex justify-between items-center"><span class="text-sm text-muted-foreground">Disbursed</span><span class="font-medium font-mono text-rose-500">-{{ formatNumber(ccStatement.reward_points_summary.disbursed) }}</span></div>
                <div v-if="ccStatement.reward_points_summary.adjusted_lapsed > 0" class="flex justify-between items-center"><span class="text-sm text-muted-foreground">Adjusted / Lapsed</span><span class="font-medium font-mono text-rose-500">-{{ formatNumber(ccStatement.reward_points_summary.adjusted_lapsed) }}</span></div>
                
                <div v-if="ccStatement.reward_points_summary.opening_balance !== 0 || ccStatement.reward_points_summary.closing_balance !== 0" class="flex justify-between items-center mt-2 border-t pt-2"><span class="text-sm font-medium">Closing Balance</span><span class="font-bold font-mono text-lg text-primary">{{ formatNumber(ccStatement.reward_points_summary.closing_balance) }}</span></div>
                <div v-if="ccStatement.reward_points_summary.expiring_in_30_days" class="flex justify-between items-center text-rose-500"><span class="text-xs">Expiring (30d)</span><span class="font-medium font-mono text-xs">{{ formatNumber(ccStatement.reward_points_summary.expiring_in_30_days) }}</span></div>
                <div v-if="ccStatement.reward_points_summary.expiring_in_60_days" class="flex justify-between items-center text-rose-500"><span class="text-xs">Expiring (60d)</span><span class="font-medium font-mono text-xs">{{ formatNumber(ccStatement.reward_points_summary.expiring_in_60_days) }}</span></div>
              </div>
            </CardContent>
          </Card>
        </div>

        <Accordion type="single" collapsible class="w-full">
          <AccordionItem value="transactions" class="border rounded-lg bg-card text-card-foreground shadow-sm overflow-hidden" :disabled="!ccStatement.transactions?.length">
            <AccordionTrigger class="group hover:no-underline px-4 py-4 data-[state=open]:border-b border-border">
              <span class="font-medium text-foreground text-lg text-left w-full pr-4">Transactions</span>
              <template #icon>
                <div class="flex items-center gap-1.5 text-xs font-mono bg-primary/10 text-primary pl-2.5 pr-2 py-1.5 rounded shrink-0 ml-2">
                  <span>{{ ccStatement.transactions?.length || 0 }} {{ ccStatement.transactions?.length === 1 ? 'Txn' : 'Txns' }}</span>
                  <ChevronDown v-if="ccStatement.transactions?.length" class="h-4 w-4 transition-transform duration-200 group-data-[state=open]:rotate-180" />
                </div>
              </template>
            </AccordionTrigger>
            <AccordionContent class="p-4">
              <div class="rounded-md border border-border overflow-x-auto">
              <Table>
                <TableHeader class="bg-muted/50">
                  <TableRow class="hover:bg-transparent">
                    <TableHead class="text-muted-foreground whitespace-nowrap">Date</TableHead>
                    <TableHead class="text-muted-foreground whitespace-nowrap">Description</TableHead>
                    <TableHead class="text-muted-foreground whitespace-nowrap">Card Name</TableHead>
                    <TableHead class="text-right text-muted-foreground whitespace-nowrap">Amount</TableHead>
                    <TableHead class="text-right text-muted-foreground whitespace-nowrap">Rewards</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  <TableRow v-for="(txn, idx) in ccStatement.transactions" :key="idx" class="hover:bg-muted/50 transition-colors">
                    <TableCell class="text-foreground whitespace-nowrap">{{ formatDateLocal(txn.date) }}</TableCell>
                    <TableCell class="text-foreground text-sm">
                      <span v-if="txn.category" class="mr-2 px-1.5 py-0.5 rounded text-[10px] font-bold bg-muted text-muted-foreground">{{ txn.category }}</span>
                      {{ txn.description }}
                    </TableCell>
                    <TableCell class="text-foreground text-xs text-muted-foreground whitespace-nowrap">{{ txn.owner }}</TableCell>
                    <TableCell class="text-right font-mono whitespace-nowrap" :class="{'text-emerald-500': txn.tx_type === 'Credit', 'text-foreground': txn.tx_type !== 'Credit'}">
                      <div class="inline-flex items-baseline justify-end">
                        <span v-if="txn.tx_type === 'Credit'">+</span>
                        <span>{{ formatCurrency(txn.amount) }}</span>
                      </div>
                    </TableCell>
                    <TableCell class="text-right font-mono text-emerald-500">{{ txn.reward_points > 0 ? '+' + txn.reward_points : (txn.reward_points || '') }}</TableCell>
                  </TableRow>
                </TableBody>
              </Table>
            </div>
            </AccordionContent>
          </AccordionItem>
        </Accordion>
      </div>

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
             :disabled="!asset.transactions?.length"
           >
               <AccordionTrigger class="group hover:no-underline px-4 py-4 data-[state=open]:border-b border-border">
                 <div class="flex flex-col w-full text-left pr-4 space-y-3">
                   <div class="flex justify-between items-start w-full">
                     <div class="flex flex-col items-start">
                       <span class="font-medium text-foreground text-lg">{{ asset.name }}</span>
                       <span class="text-sm text-muted-foreground mt-0.5">{{ asset.symbol || '-' }} <span v-if="asset.isin">| ISIN: {{ asset.isin }}</span></span>
                     </div>
                   </div>
                   
                   <div class="grid grid-cols-1 md:grid-cols-5 lg:grid-cols-6 gap-4">
                     <!-- Period Activity -->
                     <div class="flex flex-col md:col-span-2 lg:col-span-2 border-b md:border-b-0 md:border-r pb-4 md:pb-0 pr-0 md:pr-4">
                       <span class="text-xs text-muted-foreground font-semibold mb-1 uppercase tracking-wider">Period Activity</span>
                       <div class="grid grid-cols-2 gap-4 mt-1">
                         <div class="flex flex-col">
                           <span class="text-muted-foreground text-xs">Invested</span>
                           <span class="font-medium font-mono">{{ formatCurrency(asset.period_invested_value) }}</span>
                         </div>
                         <div class="flex flex-col">
                           <span class="text-muted-foreground text-xs">Units Added</span>
                           <span class="font-medium font-mono">{{ formatNumber(asset.period_units) }}</span>
                         </div>
                       </div>
                       <div v-if="asset.period_realized_value > 0" class="flex flex-col mt-2">
                         <span class="text-muted-foreground text-xs">Realized / Sold</span>
                         <span class="font-medium font-mono">{{ formatCurrency(asset.period_realized_value) }}</span>
                       </div>
                     </div>
                     
                     <!-- Overall Balance -->
                     <div class="flex flex-col md:col-span-3 lg:col-span-4 pl-0 md:pl-2">
                       <span class="text-xs text-muted-foreground font-semibold mb-1 uppercase tracking-wider">Overall Balance</span>
                       <div class="grid grid-cols-2 lg:grid-cols-4 gap-4 mt-1">
                         <div class="flex flex-col">
                           <TooltipProvider>
                             <Tooltip>
                               <TooltipTrigger class="text-muted-foreground text-xs flex items-center gap-1 cursor-help justify-start">
                                 Invested
                                 <HelpCircle class="w-3 h-3 text-muted-foreground/70" />
                               </TooltipTrigger>
                               <TooltipContent>
                                 <p>Total Cost Basis</p>
                               </TooltipContent>
                             </Tooltip>
                           </TooltipProvider>
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
                             <div v-if="asset.current_nav_date" class="flex items-baseline gap-1 mt-0.5">
                               <span class="text-[10px] text-muted-foreground">on</span>
                               <span class="font-medium font-mono text-sm">{{ formatDateLocal(asset.current_nav_date) }}</span>
                             </div>
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
                 <template #icon>
                    <div class="flex items-center gap-1.5 text-xs font-mono bg-primary/10 text-primary pl-2.5 pr-2 py-1.5 rounded shrink-0 ml-2 self-start mt-0.5">
                      <span>{{ asset.transactions?.length || 0 }} {{ asset.transactions?.length === 1 ? 'Txn' : 'Txns' }}</span>
                      <ChevronDown v-if="asset.transactions?.length" class="h-4 w-4 transition-transform duration-200 group-data-[state=open]:rotate-180" />
                    </div>
                  </template>
               </AccordionTrigger>
               <AccordionContent>
                 <div class="rounded-md border border-border mt-2 overflow-x-auto">
                   <Table>
                     <TableHeader class="bg-muted/50">
                       <TableRow class="hover:bg-transparent">
                         <TableHead class="text-muted-foreground whitespace-nowrap">Date</TableHead>
                         <TableHead class="text-muted-foreground whitespace-nowrap">Type</TableHead>
                         <TableHead class="text-muted-foreground whitespace-nowrap">Description</TableHead>
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
                         <TableCell class="text-foreground">
                            <span :class="{'text-emerald-500': txn.tx_type === 'BUY', 'text-rose-500': txn.tx_type === 'SELL'}">
                              {{ txn.tx_type || '-' }}
                            </span>
                         </TableCell>
                         <TableCell class="text-foreground text-xs">{{ txn.description || '-' }}</TableCell>
                         <TableCell class="text-right font-mono text-foreground">{{ formatCurrency(txn.amount) }}</TableCell>
                         <TableCell class="text-right font-mono text-foreground">{{ formatNumber(txn.units) }}</TableCell>
                         <TableCell class="text-right font-mono text-foreground">{{ formatCurrency(txn.nav) }}</TableCell>
                         <TableCell class="text-right font-mono text-foreground">{{ txn.fee ? formatCurrency(txn.fee) : '-' }}</TableCell>
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
