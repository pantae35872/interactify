<template>
  <div class="app">
    <RouterView />
  </div>
</template>

<script lang="ts">
//const socket: WebSocket = new WebSocket('ws://49.228.131.159:3032')
import { defineComponent, } from 'vue'
import { Network } from './network/Network';
export const network = new Network();

export default defineComponent({
  name: 'App',
  components: {},
  setup() {
    network.initialize();
  },
})
</script>

<style></style>
