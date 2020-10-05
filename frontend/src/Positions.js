import * as React from "react";

import { List } from "react-admin";
import { MoneyField, GainField } from "./MoneyUtils";
import { Datagrid, TextField } from "react-admin";

export const PositionDataGrid = (props) => (
  <Datagrid>
    <TextField source="symbol" />
    <TextField source="quantity" />
    <MoneyField source="averagePrice" />
    <MoneyField source="currentPrice" />
    <MoneyField source="costBasis" />
    <MoneyField
      source="currentValue"
      calculate={(r) => r.currentPrice * r.quantity}
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
