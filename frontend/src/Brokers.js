import * as React from "react";
import { List, Datagrid, Edit, Create, SimpleForm, DateField, TextField, EditButton, TextInput } from 'react-admin';
import BookIcon from '@material-ui/icons/Book';
export const BrokerIcon = BookIcon;

export const BrokerList = (props) => (
    <List {...props}>
        <Datagrid>
            <TextField source="name" />
            <TextField source="cnpj" />
            <EditButton basePath="/brokers" />
        </Datagrid>
    </List>
);

const BrokerName = ({ record }) => {
    return <span>Broker {record ? `"${record.name}"` : ''}</span>;
};

export const BrokerEdit = (props) => (
    <Edit title={<BrokerName />} {...props}>
        <SimpleForm>
            <TextInput disabled source="id" />
            <TextInput source="name" />
            <TextInput source="cnpj" />
        </SimpleForm>
    </Edit>
);

export const BrokerCreate = (props) => (
    <Create title="Create a Broker" {...props}>
        <SimpleForm>
            <TextInput source="name" />
            <TextInput source="cnpj" />
        </SimpleForm>
    </Create>
);