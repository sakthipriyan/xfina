<script setup>
import { ref, onMounted, computed } from 'vue';
import { useDark, useToggle } from '@vueuse/core';
import init, { parse_ibkr, parse_cams, parse_hdfc_cc, parse_icici_cc, parse_hdfc_ba, parse_icici_ba, parse_sbi_ba, parse_bob_ba } from './wasm/xfina_wasm.js';
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
import StatementHeader from '@/components/StatementHeader.vue';

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
const bankStatement = ref(null);
const equityStatement = ref(null);
const isProcessing = ref(false);
const parseTime = ref(null);

const selectedCategory = ref('Mutual Funds');
const selectedSource = ref('CAMS');
const password = ref('');

const requiresPassword = computed(() => {
    return selectedCategory.value === 'Mutual Funds' || (selectedCategory.value === 'Bank Accounts' && selectedSource.value === 'SBI');
});

const getFileFormat = computed(() => {
    if (selectedCategory.value === 'Mutual Funds') return 'PDF';
    if (selectedCategory.value === 'Bank Accounts') {
        if (selectedSource.value === 'HDFC' || selectedSource.value === 'ICICI' || selectedSource.value === 'BoB') return 'Excel';
        return 'PDF';
    }
    if (selectedCategory.value === 'Credit Cards') {
        if (selectedSource.value === 'ICICI') return 'Excel';
        return 'CSV';
    }
    if (selectedCategory.value === 'Intl Brokers') return 'CSV';
    return 'File';
});

const getAcceptString = computed(() => {
    if (selectedCategory.value === 'Mutual Funds') return '.pdf';
    if (selectedCategory.value === 'Bank Accounts') {
        if (selectedSource.value === 'HDFC' || selectedSource.value === 'ICICI' || selectedSource.value === 'BoB') return '.xls,.xlsx';
        return '.pdf';
    }
    if (selectedCategory.value === 'Credit Cards') {
        if (selectedSource.value === 'ICICI') return '.xls,.xlsx';
        return '.csv';
    }
    return '*';
});

const setCategory = (cat) => {
    selectedCategory.value = cat;
    portfolio.value = null;
    ccStatement.value = null;
    bankStatement.value = null;
    equityStatement.value = null;
    error.value = null;
    if (cat === 'Mutual Funds') selectedSource.value = 'CAMS';
    else if (cat === 'Intl Brokers') selectedSource.value = 'IBKR';
    else if (cat === 'Credit Cards') selectedSource.value = 'HDFC';
    else if (cat === 'Bank Accounts') selectedSource.value = 'HDFC';
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
    bankStatement.value = null;
    equityStatement.value = null;
    isProcessing.value = true;
    parseTime.value = null;
    
    // Yield to the event loop so the "Parsing..." UI can render
    await new Promise(resolve => setTimeout(resolve, 10));

    try {
        let jsonString;
        const start = performance.now();
        
        if (selectedCategory.value === 'Bank Accounts') {
            const arrayBuffer = await file.arrayBuffer();
            const uint8Array = new Uint8Array(arrayBuffer);
            if (selectedSource.value === 'HDFC') {
                jsonString = parse_hdfc_ba(uint8Array);
            } else if (selectedSource.value === 'ICICI') {
                jsonString = parse_icici_ba(uint8Array, file.name);
            } else if (selectedSource.value === 'SBI') {
                jsonString = parse_sbi_ba(uint8Array, password.value ? password.value : null, file.name);
            } else if (selectedSource.value === 'BoB') {
                jsonString = parse_bob_ba(uint8Array);
            }
            bankStatement.value = JSON.parse(jsonString);
        } else if (selectedSource.value === 'IBKR') {
            const text = await file.text();
            jsonString = parse_ibkr(text);
            equityStatement.value = JSON.parse(jsonString);
        } else if (selectedSource.value === 'CAMS') {
            const arrayBuffer = await file.arrayBuffer();
            const uint8Array = new Uint8Array(arrayBuffer);
            jsonString = parse_cams(uint8Array, password.value ? password.value : null);
            portfolio.value = JSON.parse(jsonString);
        } else if (selectedSource.value === 'HDFC') {
            const text = await file.text();
            jsonString = parse_hdfc_cc(text, file.name);
            ccStatement.value = JSON.parse(jsonString);
        } else if (selectedSource.value === 'ICICI') {
            const arrayBuffer = await file.arrayBuffer();
            const uint8Array = new Uint8Array(arrayBuffer);
            jsonString = parse_icici_cc(uint8Array, file.name);
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
    const formatted = Math.abs(num).toLocaleString('en-IN', { minimumFractionDigits: 2, maximumFractionDigits: 2 });
    return (num < 0 ? '-' : '') + getCurrencySymbol() + formatted;
};

const formatNumber = (val) => {
    if (val === null || val === undefined) return '-';
    return Number(val).toLocaleString('en-IN', { minimumFractionDigits: 0, maximumFractionDigits: 4 });
};

const formatDate = (ts) => {
    if (ts === null || ts === undefined || ts === '') return '-';
    const d = new Date(Number(ts) * 1000);
    if (isNaN(d)) return ts;
    return new Intl.DateTimeFormat(undefined, { 
        year: 'numeric', 
        month: 'short', 
        day: 'numeric',
        timeZone: 'UTC'
    }).format(d);
};

const formatDateTime = (ts, path = null, dateOnlyPaths = []) => {
    if (ts === null || ts === undefined || ts === '') return '-';
    const d = new Date(Number(ts) * 1000);
    if (isNaN(d)) return ts;

    const forceDateOnly = path && dateOnlyPaths && dateOnlyPaths.includes(path);

    if (!forceDateOnly) {
        return new Intl.DateTimeFormat(undefined, { 
            year: 'numeric', 
            month: 'short', 
            day: 'numeric',
            hour: '2-digit', 
            minute: '2-digit', 
            second: '2-digit',
            timeZone: 'Asia/Kolkata'
        }).format(d);
    } else {
        return new Intl.DateTimeFormat(undefined, { 
            year: 'numeric', 
            month: 'short', 
            day: 'numeric',
            timeZone: 'Asia/Kolkata'
        }).format(d);
    }
};

const hasRewards = (stmt) => {
    if (!stmt?.summary?.xfina?.rewardPointsSummary) return false;
    const s = stmt.summary.xfina.rewardPointsSummary;
    return s.openingBalance !== 0 || 
           s.earned !== 0 || 
           s.disbursed !== 0 || 
           s.adjustedLapsed !== 0 || 
           s.closingBalance !== 0 || 
           s.defaultRewards !== 0;
};

const getAssetTransactions = (holding) => {
    if (!equityStatement.value?.transactions?.transaction) return [];
    return equityStatement.value.transactions.transaction.filter(txn => 
        txn.symbol === holding.description || 
        txn.symbol === holding.issuerName || 
        txn.companyName === holding.issuerName
    );
};
</script>

<template>
  <div class="min-h-screen bg-background text-foreground p-8 font-sans transition-colors duration-200">
    <div class="max-w-6xl mx-auto space-y-8">
      
      <!-- Header -->
      <div class="flex flex-col md:flex-row md:justify-between md:items-start gap-4">
        <div class="flex items-start gap-5">
          <img src="/favicon.svg" alt="Xfina Logo" class="w-16 h-16" />
          <div class="space-y-2">
            <h1 class="text-3xl font-bold tracking-tight">Xfina<span class="text-xl font-medium text-muted-foreground ml-2">sakthipriyan.com</span></h1>
            <p class="text-muted-foreground mt-2 leading-relaxed">
              e<strong>X</strong>tract <strong>fina</strong>ncial statements entirely in your browser with Rust/Wasm<br />
              Fast, private, zero-setup, and without uploading your files to any server.
            </p>
          </div>
        </div>
        <div class="flex items-start space-x-3">
          <a href="https://github.com/sakthipriyan/xfina" target="_blank" rel="noopener noreferrer" class="no-underline">
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
            <CardTitle>Extract Statement</CardTitle>
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
          <div class="flex flex-wrap gap-4 mb-6">
            <Button 
              :variant="selectedCategory === 'Bank Accounts' ? 'default' : 'outline'"
              @click="setCategory('Bank Accounts')"
            >Bank Accounts</Button>
            <Button 
              :variant="selectedCategory === 'Credit Cards' ? 'default' : 'outline'"
              @click="setCategory('Credit Cards')"
            >Credit Cards</Button>
            <Button 
              :variant="selectedCategory === 'Mutual Funds' ? 'default' : 'outline'"
              @click="setCategory('Mutual Funds')"
            >Mutual Funds</Button>
            <Button 
              :variant="selectedCategory === 'Intl Brokers' ? 'default' : 'outline'"
              @click="setCategory('Intl Brokers')"
            >Intl Brokers</Button>
          </div>

          <div class="flex flex-col md:flex-row gap-6 items-end">
             <div class="space-y-2" v-if="selectedCategory === 'Mutual Funds'">
               <Label>Provider</Label>
               <div class="flex flex-wrap gap-4">
                 <Button :variant="selectedSource === 'CAMS' ? 'default' : 'outline'" @click="selectedSource = 'CAMS'">CAMS</Button>
               </div>
             </div>
             <div class="space-y-2" v-if="selectedCategory === 'Intl Brokers'">
               <Label>Broker</Label>
               <div class="flex flex-wrap gap-4">
                 <Button :variant="selectedSource === 'IBKR' ? 'default' : 'outline'" @click="selectedSource = 'IBKR'">IBKR</Button>
               </div>
             </div>
             <div class="space-y-2" v-if="selectedCategory === 'Credit Cards'">
               <Label>Bank</Label>
               <div class="flex flex-wrap gap-4">
                 <Button :variant="selectedSource === 'HDFC' ? 'default' : 'outline'" @click="selectedSource = 'HDFC'">HDFC Bank</Button>
                 <Button :variant="selectedSource === 'ICICI' ? 'default' : 'outline'" @click="selectedSource = 'ICICI'">ICICI Bank</Button>
               </div>
             </div>
             <div class="space-y-2" v-if="selectedCategory === 'Bank Accounts'">
               <Label>Bank</Label>
               <div class="flex flex-wrap gap-4">
                 <Button :variant="selectedSource === 'BoB' ? 'default' : 'outline'" @click="selectedSource = 'BoB'">Bank of Baroda</Button>
                 <Button :variant="selectedSource === 'HDFC' ? 'default' : 'outline'" @click="selectedSource = 'HDFC'">HDFC Bank</Button>
                 <Button :variant="selectedSource === 'ICICI' ? 'default' : 'outline'" @click="selectedSource = 'ICICI'">ICICI Bank</Button>
                 <Button :variant="selectedSource === 'SBI' ? 'default' : 'outline'" @click="selectedSource = 'SBI'">State Bank of India</Button>
               </div>
             </div>

             <div class="space-y-2 w-full md:w-auto ml-auto">
               <Label class="invisible hidden md:block">Action</Label>
               <div v-if="requiresPassword" class="flex w-full max-w-md">
                 <Input 
                    type="password" 
                    v-model="password"
                    placeholder="Password" 
                    class="rounded-r-none bg-background border-border focus-visible:z-10 focus-visible:ring-1 border-r-0"
                  />
                  <Button asChild class="rounded-l-none cursor-pointer">
                    <label>
                      <span>Import {{ getFileFormat }}</span>
                      <input type="file" class="hidden" :accept="getAcceptString" @change="onFileSelect" />
                    </label>
                  </Button>
               </div>
               <div v-else class="flex w-full max-w-md">
                  <Button asChild class="cursor-pointer w-full sm:w-auto">
                    <label>
                      <span>Import {{ getFileFormat }}</span>
                      <input type="file" class="hidden" :accept="getAcceptString" @change="onFileSelect" />
                    </label>
                  </Button>
               </div>
             </div>
          </div>
        </CardContent>
      </Card>
      <div v-else class="text-muted-foreground animate-pulse">Loading WebAssembly module...</div>
      
      <!-- Credit Card Results Table -->
      <div v-if="ccStatement" class="space-y-6">
        
        <!-- Standardized Header -->
        <StatementHeader 
          :customerName="ccStatement.profile?.holders?.holder?.[0]?.name || 'Customer'"
          :institutionName="ccStatement.xfina?.institutionName || 'Credit Card'"
          statementType="Credit Card"
          :accountNumber="ccStatement.maskedAccNumber || ''"
          :statementDetails="[
            ...(ccStatement.transactions?.startDate ? [{ label: 'From', value: formatDate(ccStatement.transactions.startDate), derived: ccStatement.transactions?.xfina?.startDateDerived }] : []),
            ...(ccStatement.transactions?.endDate ? [{ label: 'To', value: formatDate(ccStatement.transactions.endDate), derived: ccStatement.transactions?.xfina?.endDateDerived }] : []),
            ...(ccStatement.xfina?.generatedDate ? [{ label: 'Generated', value: formatDateTime(ccStatement.xfina.generatedDate, 'xfina.generatedDate', ccStatement.xfina?.dateOnlyPaths) }] : []),
            ...(ccStatement.summary?.dueDate ? [{ label: 'Due Date', value: formatDate(ccStatement.summary.dueDate) }] : [])
          ]"
        />

        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          <Card class="bg-card text-card-foreground shadow-sm">
            <CardHeader class="pb-2">
              <CardTitle class="text-sm text-muted-foreground font-semibold uppercase tracking-wider">Account Summary</CardTitle>
            </CardHeader>
            <CardContent v-if="ccStatement.summary">
              <div class="grid grid-cols-[1fr_auto_auto] gap-x-4 gap-y-2 items-center">
                <div class="col-span-3 flex justify-between items-center mb-1 border-b pb-2">
                  <span class="text-sm font-medium">Opening Balance</span>
                  <span class="font-bold font-mono text-lg text-primary">{{ formatCurrency(ccStatement.summary.xfina?.openingBalance) }}</span>
                </div>
                
                <span class="text-sm text-muted-foreground">Payments</span>
                <div class="justify-self-end">
                  <TooltipProvider>
                    <Tooltip>
                      <TooltipTrigger class="text-xs text-muted-foreground bg-muted/50 px-1.5 rounded cursor-help font-mono border border-border/50">
                        {{ ccStatement.transactions?.transaction?.filter(t => t.txnType === 'CREDIT').length || 0 }}
                      </TooltipTrigger>
                      <TooltipContent>
                        <p>Number of payments</p>
                      </TooltipContent>
                    </Tooltip>
                  </TooltipProvider>
                </div>
                <span class="font-medium font-mono text-emerald-500 text-right">+ {{ formatCurrency(ccStatement.summary.xfina?.paymentCredit) }}</span>

                <div v-if="ccStatement.summary.xfina?.ownerCreditBreakdown && Object.keys(ccStatement.summary.xfina.ownerCreditBreakdown).length > 1" class="col-span-3 pl-4 border-l-2 border-muted space-y-1 my-1">
                  <div v-for="(amount, owner) in ccStatement.summary.xfina.ownerCreditBreakdown" :key="owner" class="flex justify-between items-center">
                    <span class="text-sm text-muted-foreground truncate mr-2">{{ owner }}</span>
                    <span class="font-medium font-mono text-sm text-emerald-500">+ {{ formatCurrency(amount) }}</span>
                  </div>
                </div>
                
                <span class="text-sm text-muted-foreground">Purchases</span>
                <div class="justify-self-end">
                  <TooltipProvider>
                    <Tooltip>
                      <TooltipTrigger class="text-xs text-muted-foreground bg-muted/50 px-1.5 rounded cursor-help font-mono border border-border/50">
                        {{ ccStatement.transactions?.transaction?.filter(t => t.txnType === 'DEBIT').length || 0 }}
                      </TooltipTrigger>
                      <TooltipContent>
                        <p>Number of purchases</p>
                      </TooltipContent>
                    </Tooltip>
                  </TooltipProvider>
                </div>
                <span class="font-medium font-mono text-foreground text-right">{{ formatCurrency(ccStatement.summary.xfina?.purchasesDebits) }}</span>
                
                <div v-if="ccStatement.summary.xfina?.ownerDebitBreakdown && Object.keys(ccStatement.summary.xfina.ownerDebitBreakdown).length > 1" class="col-span-3 pl-4 border-l-2 border-muted space-y-1 my-1">
                  <div v-for="(amount, owner) in ccStatement.summary.xfina.ownerDebitBreakdown" :key="owner" class="flex justify-between items-center">
                    <span class="text-sm text-muted-foreground truncate mr-2">{{ owner }}</span>
                    <span class="font-medium font-mono text-sm text-foreground">{{ formatCurrency(amount) }}</span>
                  </div>
                </div>

                <div v-if="ccStatement.summary.financeCharges > 0" class="col-span-3 flex justify-between items-center">
                  <span class="text-sm text-muted-foreground">Finance Charges</span>
                  <span class="font-medium font-mono text-foreground">{{ formatCurrency(ccStatement.summary.financeCharges) }}</span>
                </div>
                
                <div class="col-span-3 flex justify-between items-center mt-1 border-t pt-2">
                  <span class="text-sm font-medium">Total Dues</span>
                  <span class="font-bold font-mono text-lg text-primary">{{ formatCurrency(ccStatement.summary.totalDueAmount) }}</span>
                </div>
                
                <div class="col-span-3 flex justify-between items-center mt-1">
                  <span class="text-xs text-muted-foreground">Min Amount Due</span>
                  <span class="font-medium font-mono text-xs">{{ formatCurrency(ccStatement.summary.minDueAmount) }}</span>
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
                <div class="flex justify-between items-center"><span class="text-sm text-muted-foreground">Credit Limit</span><span class="font-medium font-mono">{{ formatCurrency(ccStatement.summary?.creditLimit) }}</span></div>
                <div class="flex justify-between items-center"><span class="text-sm text-muted-foreground">Available Limit</span><span class="font-medium font-mono">{{ formatCurrency(ccStatement.summary?.availableCredit) }}</span></div>
                <div class="flex justify-between items-center"><span class="text-sm text-muted-foreground">Cash Limit</span><span class="font-medium font-mono">{{ formatCurrency(ccStatement.summary?.cashLimit) }}</span></div>
              </div>
            </CardContent>
          </Card>

          <Card v-if="hasRewards(ccStatement)" class="bg-card text-card-foreground shadow-sm">
            <CardHeader class="pb-2">
              <CardTitle class="text-sm text-muted-foreground font-semibold uppercase tracking-wider">Rewards Summary</CardTitle>
            </CardHeader>
            <CardContent>
              <div class="space-y-2">
                <div v-if="ccStatement.summary.xfina.rewardPointsSummary.openingBalance !== 0 || ccStatement.summary.xfina.rewardPointsSummary.closingBalance !== 0" class="flex justify-between items-center mb-2 border-b pb-2"><span class="text-sm font-medium">Opening Balance</span><span class="font-bold font-mono text-lg text-primary">{{ formatNumber(ccStatement.summary.xfina.rewardPointsSummary.openingBalance) }}</span></div>
                
                <div class="flex justify-between items-center"><span class="text-sm text-muted-foreground">Earned</span><span class="font-medium font-mono text-emerald-500">+{{ formatNumber(ccStatement.summary.xfina.rewardPointsSummary.earned) }}</span></div>
                
                <div v-if="ccStatement.summary.xfina.rewardPrograms && ccStatement.summary.xfina.rewardPrograms.length > 0" class="pl-4 border-l-2 border-muted space-y-1 my-1">
                  <div class="flex justify-between items-center">
                    <span class="text-sm text-muted-foreground truncate mr-2">Rewards</span>
                    <span class="font-medium font-mono text-sm text-emerald-500">+{{ formatNumber(ccStatement.summary.xfina.rewardPointsSummary.defaultRewards) }}</span>
                  </div>
                  <div v-for="(prog, idx) in ccStatement.summary.xfina.rewardPrograms" :key="idx" class="flex justify-between items-center">
                    <span class="text-sm text-muted-foreground truncate mr-2" :title="prog.program">{{ prog.program }}</span>
                    <span class="font-medium font-mono text-sm text-emerald-500">+{{ formatNumber(prog.bonusPoints) }}</span>
                  </div>
                </div>

                <div v-if="ccStatement.summary.xfina.rewardPointsSummary.disbursed > 0" class="flex justify-between items-center"><span class="text-sm text-muted-foreground">Disbursed</span><span class="font-medium font-mono text-rose-500">-{{ formatNumber(ccStatement.summary.xfina.rewardPointsSummary.disbursed) }}</span></div>
                <div v-if="ccStatement.summary.xfina.rewardPointsSummary.adjustedLapsed > 0" class="flex justify-between items-center"><span class="text-sm text-muted-foreground">Adjusted / Lapsed</span><span class="font-medium font-mono text-foreground">{{ formatNumber(ccStatement.summary.xfina.rewardPointsSummary.adjustedLapsed) }}</span></div>
                
                <div v-if="ccStatement.summary.xfina.rewardPointsSummary.openingBalance !== 0 || ccStatement.summary.xfina.rewardPointsSummary.closingBalance !== 0" class="flex justify-between items-center mt-2 border-t pt-2"><span class="text-sm font-medium">Closing Balance</span><span class="font-bold font-mono text-lg text-primary">{{ formatNumber(ccStatement.summary.xfina.rewardPointsSummary.closingBalance) }}</span></div>
                <div v-if="ccStatement.summary.xfina.rewardPointsSummary.expiringIn30Days" class="flex justify-between items-center text-rose-500"><span class="text-xs">Expiring (30d)</span><span class="font-medium font-mono text-xs">{{ formatNumber(ccStatement.summary.xfina.rewardPointsSummary.expiringIn30Days) }}</span></div>
                <div v-if="ccStatement.summary.xfina.rewardPointsSummary.expiringIn60Days" class="flex justify-between items-center text-rose-500"><span class="text-xs">Expiring (60d)</span><span class="font-medium font-mono text-xs">{{ formatNumber(ccStatement.summary.xfina.rewardPointsSummary.expiringIn60Days) }}</span></div>
              </div>
            </CardContent>
          </Card>
        </div>

        <Accordion type="single" collapsible class="w-full">
          <AccordionItem value="transactions" class="border rounded-lg bg-card text-card-foreground shadow-sm overflow-hidden" :disabled="!ccStatement.transactions?.transaction?.length">
            <AccordionTrigger class="group hover:no-underline px-4 py-4 data-[state=open]:border-b border-border">
              <span class="font-medium text-foreground text-lg text-left w-full pr-4">Transactions</span>
              <template #icon>
                <div class="flex items-center gap-1.5 text-xs font-mono bg-primary/10 text-primary pl-2.5 pr-2 py-1.5 rounded shrink-0 ml-2">
                  <span>{{ ccStatement.transactions?.transaction?.length || 0 }} {{ ccStatement.transactions?.transaction?.length === 1 ? 'Txn' : 'Txns' }}</span>
                  <ChevronDown v-if="ccStatement.transactions?.transaction?.length" class="h-4 w-4 transition-transform duration-200 group-data-[state=open]:rotate-180" />
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
                  <TableRow v-for="(txn, idx) in ccStatement.transactions?.transaction" :key="idx" class="hover:bg-muted/50 transition-colors">
                    <TableCell class="text-foreground whitespace-nowrap">{{ formatDateTime(txn.txnDate, 'transactions.transaction.txnDate', ccStatement.xfina?.dateOnlyPaths) }}</TableCell>
                    <TableCell class="text-foreground text-sm">
                      <span v-if="txn.xfina?.category" class="mr-2 px-1.5 py-0.5 rounded text-[10px] font-bold bg-muted text-muted-foreground">{{ txn.xfina.category }}</span>
                      {{ txn.narration }}
                    </TableCell>
                    <TableCell class="text-foreground text-xs text-muted-foreground whitespace-nowrap">{{ txn.xfina?.owner }}</TableCell>
                    <TableCell class="text-right font-mono whitespace-nowrap" :class="{'text-emerald-500': txn.txnType === 'CREDIT', 'text-foreground': txn.txnType !== 'CREDIT'}">
                      <div class="inline-flex items-baseline justify-end">
                        <span v-if="txn.txnType === 'CREDIT'">+</span>
                        <span>{{ formatCurrency(txn.amount) }}</span>
                      </div>
                    </TableCell>
                    <TableCell class="text-right font-mono text-emerald-500">{{ txn.xfina?.rewardPoints > 0 ? '+' + txn.xfina.rewardPoints : (txn.xfina?.rewardPoints || '') }}</TableCell>
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
        
        <!-- Standardized Header -->
        <StatementHeader 
          v-if="portfolio.investor_info"
          :customerName="portfolio.investor_info.name || 'Investor'"
          :institutionName="selectedSource"
          statementType="Mutual Funds"
          :accountNumber="portfolio.investor_info.account_number || ''"
          :statementDetails="[
            ...(portfolio.statement_start_date ? [{ label: 'From', value: formatDate(portfolio.statement_start_date) }] : []),
            ...(portfolio.statement_end_date ? [{ label: 'To', value: formatDate(portfolio.statement_end_date) }] : []),
            ...(portfolio.generated_date ? [{ label: 'Generated', value: formatDateTime(portfolio.generated_date, 'generated_date', portfolio.date_only_paths) }] : [])
          ]"
        />

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
                   
                   <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mt-2">
                     <!-- Transaction Summary Box -->
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
                         <TableCell class="text-foreground whitespace-nowrap">{{ formatDateTime(txn.date, 'transactions.date', portfolio.date_only_paths) }}</TableCell>
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

      <!-- Bank Statement Results Table -->
      <div v-if="bankStatement" class="space-y-6">
        
        <!-- Standardized Header -->
        <StatementHeader
          :customerName="bankStatement.profile?.holders?.holder?.[0]?.name || 'Customer'"
          :address="bankStatement.profile?.holders?.holder?.[0]?.address || ''"
          :customerId="bankStatement.profile?.holders?.holder?.[0]?.xfina?.customerId || ''"
          :institutionName="bankStatement.xfina?.institutionName || 'Bank'"
          statementType="Bank Account"
          :accountNumber="bankStatement.maskedAccNumber || ''"
          :statementDetails="[
            ...(bankStatement.transactions?.startDate ? [{ label: 'From', value: formatDate(bankStatement.transactions.startDate) }] : []),
            ...(bankStatement.transactions?.endDate ? [{ label: 'To', value: formatDate(bankStatement.transactions.endDate) }] : []),
            ...(bankStatement.xfina?.generatedDate ? [{ label: 'Generated', value: formatDateTime(bankStatement.xfina.generatedDate, 'xfina.generatedDate', bankStatement.xfina?.dateOnlyPaths) }] : [])
          ]"
        />

        <div class="grid grid-cols-1 lg:grid-cols-3 gap-4">
          <Card class="bg-card text-card-foreground shadow-sm h-full" v-if="bankStatement.summary?.xfina?.openingBalance !== null && bankStatement.summary?.xfina?.openingBalance !== undefined">
            <CardHeader class="pb-2">
              <CardTitle class="text-sm text-muted-foreground font-semibold uppercase tracking-wider">Transaction Summary</CardTitle>
            </CardHeader>
            <CardContent>
              <div class="grid grid-cols-[1fr_auto_auto] gap-x-4 gap-y-2 items-center">
                <div class="col-span-3 flex justify-between items-center mb-1 border-b pb-2">
                  <span class="text-sm font-medium">Opening Balance</span>
                  <span class="font-bold font-mono text-lg text-foreground">{{ formatCurrency(bankStatement.summary?.xfina?.openingBalance) }}</span>
                </div>
                
                <span class="text-sm text-muted-foreground">Deposits</span>
                <div class="justify-self-end">
                  <TooltipProvider>
                    <Tooltip>
                      <TooltipTrigger class="text-xs text-muted-foreground bg-muted/50 px-1.5 rounded cursor-help font-mono border border-border/50">
                        {{ bankStatement.transactions?.transaction?.filter(t => t.type === 'CREDIT').length || 0 }}
                      </TooltipTrigger>
                      <TooltipContent>
                        <p>Number of deposits</p>
                      </TooltipContent>
                    </Tooltip>
                  </TooltipProvider>
                </div>
                <span class="font-medium font-mono text-emerald-500 text-right">+ {{ formatCurrency(bankStatement.transactions?.transaction?.filter(t => t.type === 'CREDIT').reduce((s, t) => s + Number(t.amount || 0), 0) || 0) }}</span>
                
                <span class="text-sm text-muted-foreground">Withdrawals</span>
                <div class="justify-self-end">
                  <TooltipProvider>
                    <Tooltip>
                      <TooltipTrigger class="text-xs text-muted-foreground bg-muted/50 px-1.5 rounded cursor-help font-mono border border-border/50">
                        {{ bankStatement.transactions?.transaction?.filter(t => t.type === 'DEBIT').length || 0 }}
                      </TooltipTrigger>
                      <TooltipContent>
                        <p>Number of withdrawals</p>
                      </TooltipContent>
                    </Tooltip>
                  </TooltipProvider>
                </div>
                <span class="font-medium font-mono text-foreground text-right">{{ formatCurrency(bankStatement.transactions?.transaction?.filter(t => t.type === 'DEBIT').reduce((s, t) => s + Number(t.amount || 0), 0) || 0) }}</span>
                
                <div class="col-span-3 flex justify-between items-center mt-1 border-t pt-2">
                  <span class="text-sm font-medium">Closing Balance</span>
                  <span class="font-bold font-mono text-lg text-primary">{{ formatCurrency(bankStatement.summary?.currentBalance) }}</span>
                </div>
              </div>
            </CardContent>
          </Card>

          <!-- Account Details Card -->
          <Card class="bg-card text-card-foreground shadow-sm lg:col-span-2" v-if="bankStatement.summary?.xfina?.accountProduct || bankStatement.profile?.holders?.holder?.[0]?.nominee || bankStatement.summary?.branch || bankStatement.summary?.ifscCode || bankStatement.summary?.micrCode || bankStatement.summary?.openingDate">
            <CardHeader class="pb-2">
              <CardTitle class="text-sm text-muted-foreground font-semibold uppercase tracking-wider">Account Details</CardTitle>
            </CardHeader>
            <CardContent>
               <div class="grid grid-cols-1 sm:grid-cols-3 gap-x-8 gap-y-4 text-sm mt-1">
                 <div class="flex flex-col" v-if="bankStatement.summary?.branch">
                   <span class="text-muted-foreground text-xs uppercase tracking-wider font-semibold">Branch</span>
                   <span class="font-medium mt-0.5">{{ bankStatement.summary?.branch }}</span>
                 </div>
                 <div class="flex flex-col" v-if="bankStatement.summary?.ifscCode">
                   <span class="text-muted-foreground text-xs uppercase tracking-wider font-semibold">IFSC Code</span>
                   <span class="font-medium font-mono mt-0.5">{{ bankStatement.summary?.ifscCode }}</span>
                 </div>
                 <div class="flex flex-col" v-if="bankStatement.summary?.micrCode">
                   <span class="text-muted-foreground text-xs uppercase tracking-wider font-semibold">MICR Code</span>
                   <span class="font-medium font-mono mt-0.5">{{ bankStatement.summary?.micrCode }}</span>
                 </div>
                 <div class="flex flex-col" v-if="bankStatement.summary?.xfina?.accountProduct">
                   <span class="text-muted-foreground text-xs uppercase tracking-wider font-semibold">Product</span>
                   <span class="font-medium mt-0.5">{{ bankStatement.summary?.xfina?.accountProduct }}</span>
                 </div>
                 <div class="flex flex-col" v-if="bankStatement.summary?.openingDate">
                   <span class="text-muted-foreground text-xs uppercase tracking-wider font-semibold">Opening Date</span>
                   <span class="font-medium mt-0.5">{{ formatDate(bankStatement.summary?.openingDate) }}</span>
                 </div>
                 <div class="flex flex-col" v-if="bankStatement.profile?.holders?.holder?.[0]?.nominee">
                   <span class="text-muted-foreground text-xs uppercase tracking-wider font-semibold">Nominee</span>
                   <span class="font-medium mt-0.5">{{ bankStatement.profile?.holders?.holder?.[0]?.nominee === 'REGISTERED' ? 'Registered' : 'Not Registered' }}</span>
                 </div>
               </div>
            </CardContent>
          </Card>
        </div>

        <Accordion type="single" collapsible class="w-full">
          <AccordionItem value="transactions" class="border rounded-lg bg-card text-card-foreground shadow-sm overflow-hidden" :disabled="!bankStatement.transactions?.transaction?.length">
            <AccordionTrigger class="group hover:no-underline px-4 py-4 data-[state=open]:border-b border-border">
              <span class="font-medium text-foreground text-lg text-left w-full pr-4">Transactions</span>
              <template #icon>
                <div class="flex items-center gap-1.5 text-xs font-mono bg-primary/10 text-primary pl-2.5 pr-2 py-1.5 rounded shrink-0 ml-2">
                  <span>{{ bankStatement.transactions?.transaction?.length || 0 }} {{ bankStatement.transactions?.transaction?.length === 1 ? 'Txn' : 'Txns' }}</span>
                  <ChevronDown v-if="bankStatement.transactions?.transaction?.length" class="h-4 w-4 transition-transform duration-200 group-data-[state=open]:rotate-180" />
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
                    <TableHead class="text-right text-muted-foreground whitespace-nowrap">Amount</TableHead>
                    <TableHead class="text-right text-muted-foreground whitespace-nowrap">Balance</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  <TableRow v-for="(txn, idx) in bankStatement.transactions?.transaction" :key="idx" class="hover:bg-muted/50 transition-colors">
                    <TableCell class="font-medium whitespace-nowrap">{{ formatDateTime(txn.xfina?.parsedDate || txn.transactionTimestamp, 'transactions.transaction.transactionTimestamp', bankStatement.xfina?.dateOnlyPaths) }}</TableCell>
                    <TableCell class="text-foreground text-sm">{{ txn.narration }}</TableCell>
                    <TableCell class="text-right font-mono whitespace-nowrap" :class="{'text-emerald-500': txn.type === 'CREDIT', 'text-foreground': txn.type !== 'CREDIT'}">
                        <span v-if="txn.type === 'CREDIT'">+</span>
                        {{ formatCurrency(txn.amount) }}
                    </TableCell>
                    <TableCell class="text-right font-mono font-medium whitespace-nowrap">{{ txn.currentBalance !== null && txn.currentBalance !== undefined ? formatCurrency(txn.currentBalance) : '-' }}</TableCell>
                  </TableRow>
                </TableBody>
              </Table>
            </div>
            </AccordionContent>
          </AccordionItem>
        </Accordion>
      </div>
      
      <!-- Equity Statement Results Table -->
      <div v-if="equityStatement" class="space-y-6">
        
        <!-- Standardized Header -->
        <StatementHeader
          :customerName="equityStatement.profile?.holders?.holder?.[0]?.name || 'Customer'"
          :address="equityStatement.profile?.holders?.holder?.[0]?.address || ''"
          :customerId="equityStatement.profile?.holders?.holder?.[0]?.xfina?.customerId || ''"
          :institutionName="equityStatement.xfina?.institutionName || 'Broker'"
          statementType="Equity / Brokerage"
          :accountNumber="equityStatement.maskedAccNumber || ''"
          :statementDetails="[
            ...(equityStatement.transactions?.startDate ? [{ label: 'From', value: formatDate(equityStatement.transactions.startDate) }] : []),
            ...(equityStatement.transactions?.endDate ? [{ label: 'To', value: formatDate(equityStatement.transactions.endDate) }] : []),
            ...(equityStatement.xfina?.generatedDate ? [{ label: 'Generated', value: formatDateTime(equityStatement.xfina.generatedDate, 'xfina.generatedDate', equityStatement.xfina?.dateOnlyPaths) }] : [])
          ]"
        />

        <div class="grid grid-cols-1 gap-4">
          <Card class="bg-card text-card-foreground shadow-sm">
            <CardHeader class="pb-2 border-b mb-3">
              <CardTitle class="text-sm text-muted-foreground font-semibold uppercase tracking-wider">Portfolio Summary</CardTitle>
            </CardHeader>
            <CardContent>
              <div class="grid grid-cols-2 md:grid-cols-4 gap-6">
                <div class="flex flex-col">
                  <span class="text-xs text-muted-foreground mb-1">Total Invested</span>
                  <span class="font-medium font-mono text-xl">{{ formatCurrency(equityStatement.summary?.investmentValue) }}</span>
                </div>
                <div class="flex flex-col">
                  <span class="text-xs text-muted-foreground mb-1">Current Value</span>
                  <span class="font-medium font-mono text-xl text-primary">{{ formatCurrency(equityStatement.summary?.currentValue) }}</span>
                  <div class="flex items-baseline gap-1 mt-1 text-[10px] text-muted-foreground" v-if="equityStatement.xfina?.generatedDate">
                    <span>as of</span>
                    <span class="font-medium font-mono">{{ formatDateTime(equityStatement.xfina?.generatedDate, 'xfina.generatedDate', equityStatement.xfina?.dateOnlyPaths) }}</span>
                  </div>
                </div>
                <div class="flex flex-col">
                  <span class="text-xs text-muted-foreground mb-1">Unrealized P&L</span>
                  <span class="font-medium font-mono text-xl" 
                        :class="(equityStatement.summary?.currentValue || 0) >= (equityStatement.summary?.investmentValue || 0) ? 'text-emerald-500' : 'text-rose-500'">
                    {{ (equityStatement.summary?.currentValue || 0) >= (equityStatement.summary?.investmentValue || 0) ? '+' : '' }}{{ formatCurrency((equityStatement.summary?.currentValue || 0) - (equityStatement.summary?.investmentValue || 0)) }}
                  </span>
                </div>
                <div class="flex flex-col">
                  <span class="text-xs text-muted-foreground mb-1">Total Assets</span>
                  <span class="font-medium font-mono text-xl">{{ equityStatement.summary?.investment?.holdings?.holding?.length || 0 }}</span>
                </div>
              </div>
            </CardContent>
          </Card>
        </div>

        <Accordion type="multiple" class="w-full space-y-4">
           <AccordionItem 
             v-for="(holding, index) in equityStatement.summary?.investment?.holdings?.holding" 
             :key="index" 
             :value="`item-${index}`"
             class="border rounded-lg bg-card text-card-foreground shadow-sm overflow-hidden"
             :disabled="!getAssetTransactions(holding).length"
           >
               <AccordionTrigger class="group hover:no-underline px-4 py-4 data-[state=open]:border-b border-border">
                 <div class="flex flex-col w-full text-left pr-4 space-y-3">
                   <div class="flex justify-between items-start w-full">
                     <div class="flex flex-col items-start">
                       <span class="font-medium text-foreground text-lg">{{ holding.issuerName || holding.description }}</span>
                       <span class="text-sm text-muted-foreground mt-0.5">{{ holding.description || holding.issuerName }} <span v-if="holding.isin">| ISIN: {{ holding.isin }}</span></span>
                     </div>
                   </div>
                   
                   <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mt-2">
                     <!-- Transaction Summary Box -->
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
                 <template #icon>
                    <div class="flex items-center gap-1.5 text-xs font-mono bg-primary/10 text-primary pl-2.5 pr-2 py-1.5 rounded shrink-0 ml-2 self-start mt-0.5">
                      <span>{{ getAssetTransactions(holding).length || 0 }} {{ getAssetTransactions(holding).length === 1 ? 'Txn' : 'Txns' }}</span>
                      <ChevronDown v-if="getAssetTransactions(holding).length" class="h-4 w-4 transition-transform duration-200 group-data-[state=open]:rotate-180" />
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
                         <TableHead class="text-right text-muted-foreground whitespace-nowrap">Total Amount</TableHead>
                         <TableHead class="text-right text-muted-foreground whitespace-nowrap">Units / Qty</TableHead>
                         <TableHead class="text-right text-muted-foreground whitespace-nowrap">Rate / Price</TableHead>
                         <TableHead class="text-right text-muted-foreground whitespace-nowrap">Fees</TableHead>
                       </TableRow>
                     </TableHeader>
                     <TableBody>
                       <TableRow v-for="(txn, idx) in getAssetTransactions(holding)" :key="idx" class="hover:bg-muted/50 transition-colors">
                         <TableCell class="text-foreground whitespace-nowrap">{{ formatDateTime(txn.transactionDateTime, 'transactions.transaction.transactionDateTime', equityStatement.xfina?.dateOnlyPaths) }}</TableCell>
                         <TableCell class="text-foreground">
                            <span class="font-medium text-xs px-2 py-1 rounded" :class="{'bg-emerald-100 text-emerald-700 dark:bg-emerald-900/30 dark:text-emerald-400': txn.type === 'BUY', 'bg-rose-100 text-rose-700 dark:bg-rose-900/30 dark:text-rose-400': txn.type === 'SELL'}">
                              {{ txn.type || '-' }}
                            </span>
                         </TableCell>
                         <TableCell class="text-right font-mono text-foreground">{{ formatCurrency(txn.tradeValue) }}</TableCell>
                         <TableCell class="text-right font-mono text-foreground">{{ formatNumber(txn.units) }}</TableCell>
                         <TableCell class="text-right font-mono text-foreground">{{ formatCurrency(txn.rate) }}</TableCell>
                         <TableCell class="text-right font-mono text-foreground">{{ formatCurrency(txn.totalCharge) }}</TableCell>
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
