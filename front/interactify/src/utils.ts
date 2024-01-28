export function appendUint8Arrays(arr1: Uint8Array, arr2: Uint8Array): Uint8Array {
  const result = new Uint8Array(arr1.length + arr2.length);

  // Copy elements from arr1
  result.set(arr1, 0);

  // Copy elements from arr2, starting from the end of arr1
  result.set(arr2, arr1.length);

  return result;
}
