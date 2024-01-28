const BUFFER_SIZE = 2048;
import { appendUint8Arrays } from '../utils';

export default class Packet {
  buffer: Uint8Array
  private readpos: number
  current_size: number

  public constructor() {
    this.buffer = new Uint8Array();
    this.readpos = 0;
    this.current_size = 0;
  }

  public writeBytes(data_buffer: ArrayBuffer): Packet {
    const data = new Uint8Array(data_buffer);
    if (data.length <= BUFFER_SIZE - this.current_size) { 
      this.buffer = appendUint8Arrays(this.buffer, data);
    } else {
      throw new Error("buffer size exceed");
    }
    this.current_size += data.length;
    return this;
  }

  public writeNumber(data: number): Packet {
    const buffer = new ArrayBuffer(8);
    const view = new DataView(buffer);
    view.setFloat64(0, data, true);
    this.writeBytes(buffer);
  
    return this;
  }

  public writeString(data: string): Packet {
    const encoder = new TextEncoder();
    const data_buffer = encoder.encode(data);
    this.writeNumber(data_buffer.byteLength);
    this.writeBytes(data_buffer.buffer);

    return this;
  }

  public writeBool(data: boolean): Packet {
    const buffer = new ArrayBuffer(1);
    const view = new DataView(buffer);

    if (data) {
      view.setUint8(0, 1);
    } else {
      view.setUint8(0, 0);
    }

    this.writeBytes(buffer);

    return this;
  }

  public readBytes(length: number): ArrayBuffer {
    const read_bytes = this.buffer.slice(this.readpos, this.readpos + length).buffer;
    this.readpos += length;
    return read_bytes;
  }
  
  public readNumber(): number {
    const buffer: ArrayBuffer = this.readBytes(8);
    const view = new DataView(buffer);
    return view.getFloat64(0, true);
  }

  public readString(): string {
    const length: number = this.readNumber();
    const bytes = this.readBytes(length);
    const decoder = new TextDecoder();
    return decoder.decode(bytes);
  }

  public readBool(): boolean {
    const buffer = this.readBytes(1);
    const view = new DataView(buffer);

    if (view.getUint8(0) == 1) {
      return true;
    } else {
      return false
    }
  }

  public build(): ArrayBuffer {
    return this.buffer.buffer;
  }
} 
