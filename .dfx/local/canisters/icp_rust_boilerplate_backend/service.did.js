export const idlFactory = ({ IDL }) => {
  const DebtPayload = IDL.Record({
    'debtor' : IDL.Text,
    'amount' : IDL.Nat64,
    'creditor' : IDL.Text,
  });
  const Debt = IDL.Record({
    'id' : IDL.Nat64,
    'debtor' : IDL.Text,
    'created_at' : IDL.Nat64,
    'amount' : IDL.Nat64,
    'creditor' : IDL.Text,
  });
  const Escrow = IDL.Record({
    'debt_id' : IDL.Nat64,
    'created_at' : IDL.Nat64,
    'amount' : IDL.Nat64,
  });
  const Error = IDL.Variant({
    'InvalidInput' : IDL.Record({ 'msg' : IDL.Text }),
    'NotFound' : IDL.Record({ 'msg' : IDL.Text }),
  });
  const Result = IDL.Variant({ 'Ok' : Escrow, 'Err' : Error });
  const Result_1 = IDL.Variant({ 'Ok' : Debt, 'Err' : Error });
  const Result_2 = IDL.Variant({ 'Ok' : IDL.Nat64, 'Err' : Error });
  return IDL.Service({
    'add_debt' : IDL.Func([DebtPayload], [IDL.Opt(Debt)], []),
    'create_escrow' : IDL.Func([IDL.Nat64, IDL.Nat64], [Result], []),
    'delete_debt' : IDL.Func([IDL.Nat64], [Result_1], []),
    'delete_escrow' : IDL.Func([IDL.Nat64], [Result], []),
    'get_debt' : IDL.Func([IDL.Nat64], [Result_1], ['query']),
    'get_escrow' : IDL.Func([IDL.Nat64], [Result], ['query']),
    'release_escrow' : IDL.Func([IDL.Nat64], [Result_2], []),
    'update_debt' : IDL.Func([IDL.Nat64, DebtPayload], [Result_1], []),
    'update_escrow' : IDL.Func([IDL.Nat64, IDL.Nat64], [Result], []),
  });
};
export const init = ({ IDL }) => { return []; };
