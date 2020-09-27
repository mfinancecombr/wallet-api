import React, { Fragment } from "react";
import {
  CardContentInner,
  ChipField,
  Create,
  Datagrid,
  DateField,
  Edit,
  EditButton,
  FormDataConsumer,
  List,
  NumberInput,
  ReferenceArrayField,
  ReferenceArrayInput,
  ReferenceField,
  ReferenceInput,
  SelectArrayInput,
  SelectInput,
  SimpleForm,
  SingleFieldList,
  TextField,
  TextInput,
} from "react-admin";
import { DateTimeInput } from "react-admin-date-inputs";
import { required } from "react-admin";

export const EventList = (props) => (
  <List {...props}>
    <Datagrid>
      <DateField showTime source="time" />
      <TextField source="symbol" />
      <TextField source="detail.price" label="Price" />
      <TextField source="detail.quantity" labe="Quantity" />
      <TextField source="detail.fees" label="Fees" />
      <TextField source="detail.type" label="Type" />
      <ReferenceField
        label="Broker"
        source="detail.broker"
        reference="brokers"
        link={false}
      >
        <TextField source="name" label="Type" />
      </ReferenceField>
      <ReferenceArrayField
        label="Portfolios"
        source="detail.portfolios"
        reference="portfolios"
      >
        <SingleFieldList>
          <ChipField source="name" />
        </SingleFieldList>
      </ReferenceArrayField>
      <EditButton basePath="/events" />
    </Datagrid>
  </List>
);

const StockOperationForm = (props) => (
  <Fragment>
    <CardContentInner>
      <NumberInput label="Price" source="detail.price" validate={required()} />
    </CardContentInner>
    <CardContentInner>
      <NumberInput
        label="Quantity"
        source="detail.quantity"
        validate={required()}
      />
    </CardContentInner>
    <CardContentInner>
      <NumberInput label="Fees" source="detail.fees" />
    </CardContentInner>
    <CardContentInner>
      <SelectInput
        label="Price"
        source="detail.type"
        choices={[
          { id: "purchase", name: "Purchase" },
          { id: "sale", name: "Sale" },
        ]}
      />
    </CardContentInner>
    <CardContentInner>
      <ReferenceInput label="Broker" source="detail.broker" reference="brokers">
        <SelectInput source="detail.broker" />
      </ReferenceInput>
    </CardContentInner>
    <CardContentInner>
      <ReferenceArrayInput
        label="Portfolios"
        source="detail.portfolios"
        reference="portfolios"
      >
        <SelectArrayInput source="detail.portfolios" />
      </ReferenceArrayInput>
    </CardContentInner>
  </Fragment>
);

const StockSplitForm = (props) => (
  <Fragment>
    <CardContentInner>
      <SelectInput
        label="Split Type"
        source="detail.splitType"
        choices={[
          { id: "split", name: "Split" },
          { id: "reverse-split", name: "Reverse Split" },
        ]}
        validate={required()}
        defaultValue="Split"
      />
    </CardContentInner>
    <CardContentInner>
      <NumberInput
        label="Factor"
        source="detail.factor"
        validate={required()}
        defaultValue="1"
      />
    </CardContentInner>
  </Fragment>
);

export const EventEdit = (props) => (
  <Edit title="Event" {...props}>
    <SimpleForm>
      <TextInput disabled source="id" />
      <DateTimeInput source="time" />
      <TextInput source="symbol" validate={required()} />
      <SelectInput
        label="Type"
        source="eventType"
        choices={[
          { id: "stock-operation", name: "Stock Operation" },
          { id: "stock-split", name: "Stock Split" },
        ]}
        validate={required()}
        defaultValue="StockOperation"
      />
      <FormDataConsumer>
        {({ formData, ...rest }) =>
          formData.eventType === "stock-operation" && (
            <StockOperationForm {...props} />
          )
        }
      </FormDataConsumer>
      <FormDataConsumer>
        {({ formData, ...rest }) =>
          formData.eventType === "stock-split" && <StockSplitForm {...props} />
        }
      </FormDataConsumer>
    </SimpleForm>
  </Edit>
);

export const EventCreate = (props) => (
  <Create title="Create a stock operation" {...props}>
    <SimpleForm>
      <DateTimeInput source="time" />
      <TextInput source="symbol" validate={required()} />
      <SelectInput
        label="Type"
        source="eventType"
        choices={[
          { id: "stock-operation", name: "Stock Operation" },
          { id: "stock-split", name: "Stock Split" },
        ]}
        validate={required()}
        defaultValue="stock-operation"
      />
      <FormDataConsumer>
        {({ formData, ...rest }) =>
          formData.eventType === "stock-operation" && (
            <StockOperationForm {...props} />
          )
        }
      </FormDataConsumer>
      <FormDataConsumer>
        {({ formData, ...rest }) =>
          formData.eventType === "stock-split" && <StockSplitForm {...props} />
        }
      </FormDataConsumer>
    </SimpleForm>
  </Create>
);
