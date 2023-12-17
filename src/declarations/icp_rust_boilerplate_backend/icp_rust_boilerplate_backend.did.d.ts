import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';

export interface Debt {
  'id' : bigint,
  'debtor' : string,
  'created_at' : bigint,
  'amount' : bigint,
  'creditor' : string,
}
export interface DebtPayload {
  'debtor' : string,
  'amount' : bigint,
  'creditor' : string,
}
export type Error = { 'InvalidInput' : { 'msg' : string } } |
  { 'NotFound' : { 'msg' : string } };
export interface Escrow {
  'debt_id' : bigint,
  'created_at' : bigint,
  'amount' : bigint,
}
export type Result = { 'Ok' : Escrow } |
  { 'Err' : Error };
export type Result_1 = { 'Ok' : Debt } |
  { 'Err' : Error };
export type Result_2 = { 'Ok' : bigint } |
  { 'Err' : Error };
export interface _SERVICE {
  'add_debt' : ActorMethod<[DebtPayload], [] | [Debt]>,
  'create_escrow' : ActorMethod<[bigint, bigint], Result>,
  'delete_debt' : ActorMethod<[bigint], Result_1>,
  'delete_escrow' : ActorMethod<[bigint], Result>,
  'get_debt' : ActorMethod<[bigint], Result_1>,
  'get_escrow' : ActorMethod<[bigint], Result>,
  'release_escrow' : ActorMethod<[bigint], Result_2>,
  'update_debt' : ActorMethod<[bigint, DebtPayload], Result_1>,
  'update_escrow' : ActorMethod<[bigint, bigint], Result>,
}
