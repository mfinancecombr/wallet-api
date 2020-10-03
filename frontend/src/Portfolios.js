import * as React from "react";
import {
  List,
  Datagrid,
  Edit,
  Create,
  Show,
  SimpleForm,
  SimpleShowLayout,
  ReferenceManyField,
  TextField,
  EditButton,
  ShowButton,
  TextInput,
} from "react-admin";
import { Divider } from "@material-ui/core";
import { PositionDataGrid } from "./Positions";
import { Performance } from "./Performance";

export const PortfolioShow = (props) => {
  return (
    <Show {...props}>
      <SimpleShowLayout>
        <TextField source="name" />
        <Divider />
        <Performance portfolio={props.id} />
        <Divider />
        <ReferenceManyField
          reference="portfolios/positions"
          target="id"
          label="Positions"
        >
          <PositionDataGrid />
        </ReferenceManyField>
      </SimpleShowLayout>
    </Show>
  );
};

export const PortfolioList = (props) => (
  <List {...props}>
    <Datagrid>
      <TextField source="name" />
      <ShowButton basePath="/portfolios" />
      <EditButton basePath="/portfolios" />
    </Datagrid>
  </List>
);

const PortfolioName = ({ record }) => {
  return <span>Portfolio {record && record.name}</span>;
};

export const PortfolioEdit = (props) => (
  <Edit title={<PortfolioName />} {...props}>
    <SimpleForm>
      <TextInput disabled source="id" />
      <TextInput source="name" />
    </SimpleForm>
  </Edit>
);

export const PortfolioCreate = (props) => (
  <Create title="Create a Portfolio" {...props}>
    <SimpleForm>
      <TextInput source="name" />
    </SimpleForm>
  </Create>
);
