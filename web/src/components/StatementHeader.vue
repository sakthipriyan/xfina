<script setup>
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Download, FileJson } from 'lucide-vue-next';

const props = defineProps({
  // Customer Card
  customerName: {
    type: String,
    required: true
  },
  institutionName: {
    type: String,
    default: ''
  },
  accountNumber: {
    type: String,
    default: ''
  },
  
  // Statement Card (Array of { label, value })
  statementDetails: {
    type: Array,
    default: () => []
  }
});
</script>

<template>
  <div class="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
    <!-- Customer/Investor Card -->
    <Card class="bg-card text-card-foreground shadow-sm flex flex-col h-full">
      <CardHeader class="pb-2">
        <CardTitle class="text-xl">{{ customerName }}</CardTitle>
      </CardHeader>
      <CardContent class="mt-auto">
        <div class="flex flex-col gap-1">
          <span class="text-sm font-medium" v-if="institutionName">{{ institutionName }}</span>
          <span class="text-sm text-muted-foreground font-mono" v-if="accountNumber">Account: {{ accountNumber }}</span>
        </div>
      </CardContent>
    </Card>

    <!-- Statement Card -->
    <Card class="bg-card text-card-foreground shadow-sm flex flex-col h-full">
      <CardHeader class="pb-2">
        <CardTitle class="text-sm text-muted-foreground font-semibold uppercase tracking-wider">Statement Details</CardTitle>
      </CardHeader>
      <CardContent>
        <table class="w-full text-sm">
          <tbody>
            <tr v-for="(detail, idx) in statementDetails" :key="idx" class="border-0">
              <td class="py-1 text-muted-foreground font-medium align-top pr-4">{{ detail.label }}</td>
              <td class="py-1 text-foreground font-medium whitespace-nowrap text-right sm:text-left">{{ detail.value }}</td>
            </tr>
          </tbody>
        </table>
      </CardContent>
    </Card>

    <!-- Export Card -->
    <Card class="bg-card text-card-foreground shadow-sm flex flex-col h-full border-dashed border-muted-foreground/30">
      <CardHeader class="pb-2">
        <CardTitle class="text-sm text-muted-foreground font-semibold uppercase tracking-wider">Export Data</CardTitle>
      </CardHeader>
      <CardContent class="flex flex-col gap-3 justify-center flex-1">
        <Button variant="outline" class="w-full justify-start gap-2" disabled title="Export to CSV (Coming Soon)">
          <Download class="h-4 w-4" />
          Export CSV
        </Button>
        <Button variant="outline" class="w-full justify-start gap-2" disabled title="Export to JSON (Coming Soon)">
          <FileJson class="h-4 w-4" />
          Export JSON
        </Button>
      </CardContent>
    </Card>
  </div>
</template>
