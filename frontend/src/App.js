import * as React from "react";
import { Admin, Resource } from "react-admin";
import jsonServerProvider from "ra-data-json-server";

import { BrokerList, BrokerEdit, BrokerCreate } from "./Brokers";
import {
  StockOperationCreate,
  StockOperationEdit,
  StockOperationList,
} from "./StockOperations";

const dataProvider = jsonServerProvider("http://localhost:8000/api/v1");
const App = () => (
  <Admin dataProvider={dataProvider}>
    <Resource
      name="brokers"
      list={BrokerList}
      edit={BrokerEdit}
      create={BrokerCreate}
    />
    <Resource
      name="stocks/operations"
      list={StockOperationList}
      edit={StockOperationEdit}
      create={StockOperationCreate}
    />
  </Admin>
);

export default App;
