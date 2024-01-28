<template>
  <div>
    <h1>Home Page</h1>
    <button @click="btn_click()">click me</button>
  </div>
</template>

<script lang="ts">
import { defineComponent, } from 'vue'
import { network } from '../App.vue';
import Packet from '../types/Packet';

export default defineComponent({
  methods: {
    btn_click() {
      network.socket.send(new Packet().writeString("Hello from client aaa").writeNumber(-50.5).build());
    }
  }
})
</script>
