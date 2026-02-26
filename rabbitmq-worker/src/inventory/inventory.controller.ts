import { Controller } from '@nestjs/common';
import { MessagePattern } from '@nestjs/microservices';

import type { InventoryProductRequest, InventoryReserveRequest } from './inventory.interfaces';

@Controller()
export class InventoryController {
  private stock = {
    Laptop: 10,
    Phone: 20,
  };

  @MessagePattern('inventory.check')
  checkInventory(data: InventoryProductRequest) {
    return this.stock[data.product] >= data.quantity;
  }

  @MessagePattern('inventory.reserve')
  reserve(data: InventoryReserveRequest) {
    if (this.stock[data.product] >= data.quantity) {
      this.stock[data.product] -= data.quantity;
      return true;
    }

    return false;
  }

  @MessagePattern('order.status')
  status(data: any) {
    return { id: data.id, status: 'CREATED' };
  }
}
