import { VueCookieNext } from "vue-cookie-next";
import Packet from '../types/Packet';

export class Network {
  socket: WebSocket
  handler: ((this: Network,packet: Packet) => void) 

  public constructor() {
    this.socket = new WebSocket('ws://49.228.131.159:3032');
    this.socket.binaryType = 'arraybuffer';
    this.handler = (packet) => {
      console.log(packet.readString());
    };
  }

  public initialize() {
    this.socket.onopen = () => {
      this.socket.send(new Packet().writeString("Hello from client aaa").writeNumber(50).build());
    }
    this.socket.onmessage = (event) => {
      const packet = new Packet().writeBytes(event.data);
      
      this.handler.call(this, packet);
    }
  }
}
