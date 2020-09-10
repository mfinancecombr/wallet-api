import * as React from "react";
import {
  List,
  Datagrid,
  Edit,
  Create,
  SimpleForm,
  TextField,
  EditButton,
  TextInput,
} from "react-admin";
import BookIcon from "@material-ui/icons/Book";
export const PortfolioIcon = BookIcon;

export const PortfolioList = (props) => (
  <List {...props}>
    <Datagrid>
      <TextField source="name" />
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
