
export interface InventoryProductRequest {
  product: string,

  quantity: number,
}

export interface InventoryReserveRequest extends InventoryProductRequest {
  orderId: string,
}
