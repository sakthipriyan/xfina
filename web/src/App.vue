<template>
  <div class="app-container">
    <h1>financial-extract WebAssembly PoC</h1>
    
    <div class="upload-zone" v-if="wasmLoaded">
      <Select v-model="selectedSource" :options="sources" optionLabel="label" placeholder="Select Source" class="w-full md:w-14rem mb-3" />
      <br />
      <FileUpload mode="basic" name="demo[]" accept=".csv" :maxFileSize="10000000" @select="onFileSelect" customUpload auto chooseLabel="Select IBKR CSV" />
    </div>
    <div v-else>
      Loading WebAssembly module...
    </div>

    <div v-if="portfolio" class="results mt-5">
      <h2>Extracted Portfolio</h2>
      <div v-for="asset in portfolio.assets" :key="asset.symbol" class="asset-card">
        <h3>{{ asset.name }} ({{ asset.symbol }})</h3>
        <DataTable :value="asset.transactions" tableStyle="min-width: 50rem">
            <Column field="date" header="Date"></Column>
            <Column field="tx_type" header="Type"></Column>
            <Column field="amount" header="Amount"></Column>
            <Column field="units" header="Units"></Column>
        </DataTable>
      </div>
    </div>
    <div v-if="error" class="error mt-3">
      {{ error }}
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue';
import init, { parse_ibkr } from './wasm/financial_extract_wasm.js';

// PrimeVue components
import Select from 'primevue/select';
import FileUpload from 'primevue/fileupload';
import DataTable from 'primevue/datatable';
import Column from 'primevue/column';

const wasmLoaded = ref(false);
const error = ref(null);
const portfolio = ref(null);
const selectedSource = ref({ label: 'IBKR (CSV)', value: 'ibkr' });
const sources = ref([
    { label: 'IBKR (CSV)', value: 'ibkr' },
    { label: 'CAMS (PDF) - Coming Soon', value: 'cams', disabled: true }
]);

onMounted(async () => {
    try {
        await init();
        wasmLoaded.value = true;
    } catch (e) {
        error.value = "Failed to load WASM module: " + e.message;
    }
});

const onFileSelect = async (event) => {
    const file = event.files[0];
    if (!file) return;
    
    error.value = null;
    portfolio.value = null;
    try {
        const text = await file.text();
        const jsonString = parse_ibkr(text);
        portfolio.value = JSON.parse(jsonString);
    } catch (e) {
        error.value = "Error parsing file: " + e;
    }
};
</script>

<style scoped>
.app-container {
  max-width: 900px;
  margin: 0 auto;
  padding: 2rem;
  font-family: 'Inter', sans-serif;
}
.upload-zone {
  padding: 2rem;
  border: 2px dashed #cbd5e1;
  border-radius: 8px;
  text-align: center;
  background-color: #f8fafc;
}
.asset-card {
  margin-bottom: 2rem;
  padding: 1rem;
  background: white;
  border-radius: 8px;
  box-shadow: 0 4px 6px -1px rgb(0 0 0 / 0.1);
}
.error {
  color: #ef4444;
  font-weight: bold;
}
.mt-3 { margin-top: 1rem; }
.mt-5 { margin-top: 2rem; }
.mb-3 { margin-bottom: 1rem; }
</style>
