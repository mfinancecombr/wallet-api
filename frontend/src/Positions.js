import * as React from "react";

import { List } from "react-admin";
import { MoneyField, GainField } from "./MoneyUtils";
import { Datagrid, TextField } from "react-admin";

export const PositionDataGrid = (props) => (
  <Datagrid>
    <TextField source="symbol" />
    <TextField source="quantity" />
    <MoneyField source="average_price" />
    <MoneyField source="current_price" />
    <MoneyField source="cost_basis" />
    <MoneyField
      source="current_value"
      calculate={(r) => r.current_price * r.quantity}
    />
    <GainField source="gain" />
  </Datagrid>
);

export const PositionList = (props) => {
  return (
    <List {...props}>
      <PositionDataGrid />
    </List>
  );
};
