import { randomUUID } from 'node:crypto';

import { Body, Controller, Get, Headers, Inject, MessageEvent, Param, Post } from '@nestjs/common';
import { ClientProxy, RmqRecordBuilder } from '@nestjs/microservices';

import { firstValueFrom, map, Observable } from 'rxjs';

import type { InventoryProductRequest, InventoryReserveRequest } from '../inventory/inventory.interfaces'
import type { CreateOrderRequest } from './orders.interfaces'

@Controller('orders')
export class OrdersController {
  constructor(@Inject('WORKER_SERVICE') private client: ClientProxy) {}

  async onApplicationBootstrap() {
    try {
      await this.client.connect();
    } catch(err) {
      console.error(err);
    }
  }


  @Post()
  async createOrder(@Headers() headers: Record<string, string>, @Body() body: CreateOrderRequest) {
    const { product, quantity } = body;

    const order = {
      id: randomUUID(),
      status: 'CREATED',
      product,
      quantity,
    };

    const inventoryRecord = new RmqRecordBuilder<InventoryProductRequest>({ product, quantity })
      .setOptions({ headers })
      .build();

    // RPC → Check inventory
    const available = await firstValueFrom<boolean>(
      this.client.send('inventory.check', inventoryRecord),
    );

    if (!available) {
      return { message: 'Not enough stock' };
    }

    const inventoryReserveRecord = new RmqRecordBuilder<InventoryReserveRequest>({
        orderId: order.id,
        product,
        quantity,
      })
      .setOptions({ headers })
      .build();

    const reserved = await firstValueFrom<boolean>(
      this.client.send('inventory.reserve', inventoryReserveRecord),
    );

    if (!reserved) {
      return { message: 'Ops just ran out' };
    }

    const orderRecord = new RmqRecordBuilder(order)
      .setOptions({ headers })
      .build();

    // Emit Event
    this.client.emit('order.created', orderRecord);

    return order;
  }

  @Post(':id/cancel')
  async cancel(@Headers() headers: Record<string, string>, @Param('id') id: string) {
    const record = new RmqRecordBuilder({ id })
      .setOptions({ headers })
      .build();
    this.client.emit('order.cancelled', record);
    return { message: `Order ${id} cancelled` };
  }

  @Get(':id/status')
  async getStatus(@Headers() headers: Record<string, string>, @Param('id') id: string) {
    const record = new RmqRecordBuilder({ id })
      .setOptions({ headers })
      .build();
    return firstValueFrom(
      this.client.send('order.status', record),
    );
  }
}
